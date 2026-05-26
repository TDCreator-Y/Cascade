use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;

pub struct CascadeConfig {
    pub vpn_port: u16,
    pub isp_ip: String,
    pub isp_port: u16,
    pub username: String,
    pub password: String,
}

pub async fn start_server(
    listener: TcpListener,
    config: Arc<CascadeConfig>,
    log_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = log_tx.send("Cascade Engine 已启动，监听 127.0.0.1:10808".to_string()).await;

    loop {
        let (client, addr) = listener.accept().await?;
        let cfg = Arc::clone(&config);
        let log = log_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(client, cfg, log.clone()).await {
                let err_str = e.to_string();
                if !err_str.contains("Connection reset by peer")
                    && !err_str.contains("Broken pipe")
                    && !err_str.contains("10053")
                {
                    let _ = log.send(format!("[错误] {} → {}", addr, e)).await;
                }
            }
        });
    }
}

async fn handle_client(
    mut client: TcpStream,
    config: Arc<CascadeConfig>,
    log_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut protocol_buf = [0u8; 1];
    let n = client.peek(&mut protocol_buf).await?;
    if n == 0 {
        return Ok(());
    }

    let is_socks5 = protocol_buf[0] == 0x05;
    let is_http = protocol_buf[0].is_ascii_uppercase();

    if !is_socks5 && !is_http {
        return Ok(());
    }

    let host: String;
    let port: u16;
    let mut dest_addr = Vec::new();
    let mut is_connect = false;
    let mut initial_data_to_target = Vec::new();

    if is_socks5 {
        let mut buf = [0u8; 2];
        client.read_exact(&mut buf).await?;
        let nmethods = buf[1] as usize;
        let mut methods = vec![0u8; nmethods];
        client.read_exact(&mut methods).await?;

        client.write_all(&[0x05, 0x00]).await?;

        let mut req_header = [0u8; 4];
        client.read_exact(&mut req_header).await?;
        if req_header[1] != 0x01 {
            return Err("Only CONNECT command is supported".into());
        }

        let atyp = req_header[3];
        dest_addr.extend_from_slice(&req_header);

        match atyp {
            0x01 => {
                let mut addr = [0u8; 6];
                client.read_exact(&mut addr).await?;
                dest_addr.extend_from_slice(&addr);
                host = format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]);
                port = u16::from_be_bytes([addr[4], addr[5]]);
            }
            0x03 => {
                let mut len = [0u8; 1];
                client.read_exact(&mut len).await?;
                dest_addr.push(len[0]);
                let mut addr = vec![0u8; len[0] as usize + 2];
                client.read_exact(&mut addr).await?;
                dest_addr.extend_from_slice(&addr);
                let domain_bytes = &addr[..len[0] as usize];
                host = String::from_utf8_lossy(domain_bytes).to_string();
                port = u16::from_be_bytes([addr[len[0] as usize], addr[len[0] as usize + 1]]);
            }
            0x04 => {
                let mut addr = [0u8; 18];
                client.read_exact(&mut addr).await?;
                dest_addr.extend_from_slice(&addr);
                let mut ip_bytes = [0u8; 16];
                ip_bytes.copy_from_slice(&addr[0..16]);
                host = std::net::Ipv6Addr::from(ip_bytes).to_string();
                port = u16::from_be_bytes([addr[16], addr[17]]);
            }
            _ => return Err("Unsupported address type".into()),
        }
    } else {
        let mut header = Vec::new();
        let mut buf = [0u8; 1];
        loop {
            if client.read_exact(&mut buf).await.is_err() {
                return Ok(());
            }
            header.push(buf[0]);
            if header.ends_with(b"\r\n\r\n") {
                break;
            }
            if header.len() > 8192 {
                return Ok(());
            }
        }

        let header_str = String::from_utf8_lossy(&header);
        let first_line = header_str.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(());
        }

        let method = parts[0];
        let uri = parts[1];

        if method == "CONNECT" {
            is_connect = true;
            let mut hp_parts = uri.split(':');
            host = hp_parts.next().unwrap_or("").to_string();
            port = hp_parts.next().unwrap_or("443").parse().unwrap_or(443);
        } else {
            is_connect = false;
            initial_data_to_target = header.clone();

            if uri.starts_with("http://") {
                let without_scheme = &uri[7..];
                let host_port_path: Vec<&str> = without_scheme.splitn(2, '/').collect();
                let mut hp_parts = host_port_path[0].split(':');
                host = hp_parts.next().unwrap_or("").to_string();
                port = hp_parts.next().unwrap_or("80").parse().unwrap_or(80);
            } else {
                let mut found_host = String::new();
                for line in header_str.lines() {
                    if line.to_lowercase().starts_with("host:") {
                        found_host = line[5..].trim().to_string();
                        break;
                    }
                }
                let mut hp_parts = found_host.split(':');
                host = hp_parts.next().unwrap_or("").to_string();
                port = hp_parts.next().unwrap_or("80").parse().unwrap_or(80);
            }
        }

        dest_addr.push(0x05);
        dest_addr.push(0x01);
        dest_addr.push(0x00);
        dest_addr.push(0x03);
        let host_bytes = host.as_bytes();
        dest_addr.push(host_bytes.len() as u8);
        dest_addr.extend_from_slice(host_bytes);
        dest_addr.extend_from_slice(&port.to_be_bytes());
    }

    // ==========================================
    // 智能分流：国内域名直连，海外走级联
    // ==========================================
    let is_direct = is_direct_domain(&host);

    if is_direct {
        let _ = log_tx.send(format!("[直连] {}:{}", host, port)).await;
        let target_addr = format!("{}:{}", host, port);

        let mut target_stream = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            TcpStream::connect(&target_addr),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                let _ = log_tx.send(format!("[错误] 直连 {} 失败: {}", target_addr, e)).await;
                reply_error(&mut client, is_socks5, is_connect).await?;
                return Err(e.into());
            }
            Err(_) => {
                let _ = log_tx.send(format!("[错误] 直连 {} 超时", target_addr)).await;
                reply_error(&mut client, is_socks5, is_connect).await?;
                return Err("Connection timeout".into());
            }
        };

        if is_socks5 {
            reply_success(&mut client, true).await?;
        } else if is_connect {
            reply_success(&mut client, false).await?;
        } else {
            target_stream.write_all(&initial_data_to_target).await?;
        }

        tokio::io::copy_bidirectional(&mut client, &mut target_stream).await?;
        return Ok(());
    }

    let _ = log_tx.send(format!("[级联] {}:{}", host, port)).await;

    // ==========================================
    // 连接本地 VPN 端口
    // ==========================================
    let vpn_addr = format!("127.0.0.1:{}", config.vpn_port);
    let mut vpn_stream = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        TcpStream::connect(&vpn_addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            let _ = log_tx.send(format!("[错误] VPN 端口 {} 连接失败: {}", config.vpn_port, e)).await;
            reply_error(&mut client, is_socks5, is_connect).await?;
            return Err(e.into());
        }
        Err(_) => {
            let _ = log_tx.send(format!("[错误] VPN 端口 {} 连接超时", config.vpn_port)).await;
            reply_error(&mut client, is_socks5, is_connect).await?;
            return Err("VPN Connection timeout".into());
        }
    };

    // ==========================================
    // 通过 VPN 建立通往远程 ISP 的隧道
    // ==========================================
    vpn_stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut vpn_resp = [0u8; 2];
    vpn_stream.read_exact(&mut vpn_resp).await?;
    if vpn_resp[0] != 0x05 || vpn_resp[1] != 0x00 {
        return Err("VPN Socks5 initial auth failed".into());
    }

    let ip_parts: Vec<u8> = config
        .isp_ip
        .split('.')
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();
    if ip_parts.len() != 4 {
        return Err("Invalid IPv4 address format for ISP IP".into());
    }
    let port_bytes = config.isp_port.to_be_bytes();

    let mut isp_tunnel_req = vec![0x05, 0x01, 0x00, 0x01];
    isp_tunnel_req.extend_from_slice(&ip_parts);
    isp_tunnel_req.extend_from_slice(&port_bytes);
    vpn_stream.write_all(&isp_tunnel_req).await?;

    let mut vpn_rep = [0u8; 4];
    vpn_stream.read_exact(&mut vpn_rep).await?;
    if vpn_rep[1] != 0x00 {
        return Err("VPN failed to connect to remote ISP".into());
    }
    skip_socks5_addr(&mut vpn_stream, vpn_rep[3]).await?;

    // ==========================================
    // 在隧道上对 ISP 进行二次 SOCKS5 认证
    // ==========================================
    vpn_stream.write_all(&[0x05, 0x01, 0x02]).await?;
    let mut isp_resp = [0u8; 2];
    vpn_stream.read_exact(&mut isp_resp).await?;
    if isp_resp[1] != 0x02 {
        return Err("Remote ISP did not accept Auth method 0x02".into());
    }

    let user = config.username.as_bytes();
    let pass = config.password.as_bytes();
    let mut auth_req = vec![0x01, user.len() as u8];
    auth_req.extend_from_slice(user);
    auth_req.push(pass.len() as u8);
    auth_req.extend_from_slice(pass);
    vpn_stream.write_all(&auth_req).await?;

    let mut auth_resp = [0u8; 2];
    vpn_stream.read_exact(&mut auth_resp).await?;
    if auth_resp[1] != 0x00 {
        return Err("Remote ISP authentication failed".into());
    }

    // ==========================================
    // 将最终目标地址发给 ISP
    // ==========================================
    vpn_stream.write_all(&dest_addr).await?;
    let mut isp_rep = [0u8; 4];
    vpn_stream.read_exact(&mut isp_rep).await?;
    if isp_rep[1] != 0x00 {
        return Err("Remote ISP failed to connect to target".into());
    }
    skip_socks5_addr(&mut vpn_stream, isp_rep[3]).await?;

    if is_socks5 {
        reply_success(&mut client, true).await?;
    } else if is_connect {
        reply_success(&mut client, false).await?;
    } else {
        vpn_stream.write_all(&initial_data_to_target).await?;
    }

    tokio::io::copy_bidirectional(&mut client, &mut vpn_stream).await?;
    Ok(())
}

/// 判断是否应直连的域名（国内流量）
pub fn is_direct_domain(host: &str) -> bool {
    // 精确匹配的特殊地址
    const DIRECT_EXACT: &[&str] = &["127.0.0.1", "localhost", "::1"];
    // 以 . 开头的 TLD 后缀（如 .cn），用 ends_with 匹配
    const DIRECT_TLD_SUFFIXES: &[&str] = &[".cn"];
    // 国内域名：精确匹配或子域名匹配（host == d 或 host.ends_with(".d")）
    const DIRECT_DOMAINS: &[&str] = &[
        "baidu.com",
        "qq.com",
        "bilibili.com",
        "taobao.com",
        "jd.com",
        "weibo.com",
        "alipay.com",
        "aliyuncs.com",
        "tencentcloud.com",
    ];

    if DIRECT_EXACT.iter().any(|&d| host == d) {
        return true;
    }
    if DIRECT_TLD_SUFFIXES.iter().any(|&s| host.ends_with(s)) {
        return true;
    }
    DIRECT_DOMAINS.iter().any(|&d| host == d || host.ends_with(&format!(".{}", d)))
}

async fn reply_success(client: &mut TcpStream, is_socks5: bool) -> Result<(), std::io::Error> {
    if is_socks5 {
        client.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
    } else {
        client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
    }
    Ok(())
}

async fn reply_error(
    client: &mut TcpStream,
    is_socks5: bool,
    _is_connect: bool,
) -> Result<(), std::io::Error> {
    if is_socks5 {
        client.write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
    } else {
        client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
    }
    Ok(())
}

async fn skip_socks5_addr(
    stream: &mut TcpStream,
    atyp: u8,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match atyp {
        0x01 => {
            let mut buf = [0u8; 6];
            stream.read_exact(&mut buf).await?;
        }
        0x03 => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len).await?;
            let mut buf = vec![0u8; len[0] as usize + 2];
            stream.read_exact(&mut buf).await?;
        }
        0x04 => {
            let mut buf = [0u8; 18];
            stream.read_exact(&mut buf).await?;
        }
        _ => return Err("Invalid ATYP received from proxy".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_domain_localhost() {
        assert!(is_direct_domain("localhost"));
        assert!(is_direct_domain("127.0.0.1"));
        assert!(is_direct_domain("::1"));
    }

    #[test]
    fn test_direct_domain_cn_tld() {
        assert!(is_direct_domain("example.cn"));
        assert!(is_direct_domain("sub.example.cn"));
        assert!(is_direct_domain("gov.cn"));
    }

    #[test]
    fn test_direct_domain_known_cn_sites() {
        assert!(is_direct_domain("baidu.com"));
        assert!(is_direct_domain("www.baidu.com"));
        assert!(is_direct_domain("map.baidu.com"));
        assert!(is_direct_domain("qq.com"));
        assert!(is_direct_domain("bilibili.com"));
        assert!(is_direct_domain("taobao.com"));
        assert!(is_direct_domain("jd.com"));
        assert!(is_direct_domain("weibo.com"));
    }

    #[test]
    fn test_cascade_domain_overseas() {
        assert!(!is_direct_domain("google.com"));
        assert!(!is_direct_domain("github.com"));
        assert!(!is_direct_domain("twitter.com"));
        assert!(!is_direct_domain("youtube.com"));
        assert!(!is_direct_domain("netflix.com"));
        assert!(!is_direct_domain("openai.com"));
        assert!(!is_direct_domain("anthropic.com"));
    }

    #[test]
    fn test_direct_domain_no_false_positive() {
        // 确保子域名不误匹配
        assert!(!is_direct_domain("notbaidu.com"));
        assert!(!is_direct_domain("fakeqq.com"));
        // .cn 后缀匹配需以 . 分隔
        assert!(!is_direct_domain("xcn.com"));
    }
}
