use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct CascadeConfig {
    pub vpn_port: u16,
    pub isp_ip: String,
    pub isp_port: u16,
    pub username: String,
    pub password: String,
}

pub async fn start_server(config: Arc<CascadeConfig>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:10808").await?;
    println!("Cascade Engine: Mixed Port (SOCKS5/HTTP) listening on 127.0.0.1:10808");

    loop {
        let (client, addr) = listener.accept().await?;
        
        let cfg = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = handle_client(client, cfg).await {
                // Suppress common connection reset errors to avoid spam
                let err_str = e.to_string();
                if !err_str.contains("Connection reset by peer") && !err_str.contains("Broken pipe") && !err_str.contains("10053") {
                    eprintln!("Cascade Engine Error [{}]: {}", addr, e);
                }
            }
        });
    }
}

async fn handle_client(mut client: TcpStream, config: Arc<CascadeConfig>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut protocol_buf = [0u8; 1];
    // 偷看第一个字节以判断协议
    let n = client.peek(&mut protocol_buf).await?;
    if n == 0 {
        return Ok(());
    }

    let is_socks5 = protocol_buf[0] == 0x05;
    // 如果首字母是 ASCII 大写字母，我们认为是 HTTP 代理请求 (CONNECT, GET, POST, OPTIONS 等)
    let is_http = protocol_buf[0].is_ascii_uppercase();

    if !is_socks5 && !is_http {
        // 静默丢弃不支持的协议
        return Ok(());
    }

    let host: String;
    let port: u16;
    let mut dest_addr = Vec::new();
    
    // 用于区分是否为 HTTPS 的 CONNECT 请求，或者是普通 HTTP 请求
    let mut is_connect = false;
    // 如果是普通 HTTP 请求，我们需要将读取的 HTTP Header 原封不动地发送给目标服务器
    let mut initial_data_to_target = Vec::new();

    if is_socks5 {
        // ==========================================
        // 1a. 本地 SOCKS5 握手阶段
        // ==========================================
        let mut buf = [0u8; 2];
        client.read_exact(&mut buf).await?;
        let nmethods = buf[1] as usize;
        let mut methods = vec![0u8; nmethods];
        client.read_exact(&mut methods).await?;
        
        // 回复无须认证
        client.write_all(&[0x05, 0x00]).await?;

        let mut req_header = [0u8; 4];
        client.read_exact(&mut req_header).await?;
        if req_header[1] != 0x01 {
            return Err("Only CONNECT command is supported".into());
        }

        let atyp = req_header[3];
        dest_addr.extend_from_slice(&req_header);

        match atyp {
            0x01 => { // IPv4
                let mut addr = [0u8; 6];
                client.read_exact(&mut addr).await?;
                dest_addr.extend_from_slice(&addr);
                host = format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]);
                port = u16::from_be_bytes([addr[4], addr[5]]);
            }
            0x03 => { // 域名
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
            0x04 => { // IPv6
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
        // ==========================================
        // 1b. 本地 HTTP/CONNECT 代理握手阶段
        // ==========================================
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
                return Ok(()); // Header too large, drop silently
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
            // HTTPS 代理请求 (CONNECT domain.com:443 HTTP/1.1)
            is_connect = true;
            let mut hp_parts = uri.split(':');
            host = hp_parts.next().unwrap_or("").to_string();
            port = hp_parts.next().unwrap_or("443").parse().unwrap_or(443);
        } else {
            // 普通 HTTP 代理请求 (GET http://example.com/ HTTP/1.1)
            is_connect = false;
            initial_data_to_target = header.clone(); // 记录下 Header 以便稍后原封不动地转发

            if uri.starts_with("http://") {
                let without_scheme = &uri[7..];
                let host_port_path: Vec<&str> = without_scheme.splitn(2, '/').collect();
                let mut hp_parts = host_port_path[0].split(':');
                host = hp_parts.next().unwrap_or("").to_string();
                port = hp_parts.next().unwrap_or("80").parse().unwrap_or(80);
            } else {
                // 如果 URI 中没有完整路径，尝试从 Host 头中提取
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

        // 构建兼容 SOCKS5 的 dest_addr 供后续级联隧道使用
        dest_addr.push(0x05);
        dest_addr.push(0x01);
        dest_addr.push(0x00);
        dest_addr.push(0x03); // 域名类型
        
        let host_bytes = host.as_bytes();
        dest_addr.push(host_bytes.len() as u8);
        dest_addr.extend_from_slice(host_bytes);
        dest_addr.extend_from_slice(&port.to_be_bytes());
    }

    // ==========================================
    // 1.5 智能分流 (Routing Engine)
    // ==========================================
    let direct_domains = vec![
        ".cn", "baidu.com", "qq.com", "bilibili.com", "taobao.com", "127.0.0.1", "localhost", "github.com"
    ];
    let is_direct = direct_domains.iter().any(|d| host.ends_with(d) || host == *d);

    if is_direct {
        println!("Routing Engine: Direct Connect -> {}:{}", host, port);
        let target_addr = format!("{}:{}", host, port);
        
        // 前置连通性校验与异常处理
        let mut target_stream = match tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(&target_addr)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                eprintln!("Error: Target port {} connectivity check failed for {}: {}", port, host, e);
                reply_error(&mut client, is_socks5, is_connect).await?;
                return Err(e.into());
            }
            Err(_) => {
                eprintln!("Error: Target port {} connection timeout for {}", port, host);
                reply_error(&mut client, is_socks5, is_connect).await?;
                return Err("Connection timeout".into());
            }
        };
        
        if is_socks5 {
            reply_success(&mut client, true).await?;
        } else if is_connect {
            reply_success(&mut client, false).await?;
        } else {
            // 普通 HTTP 请求，不回复 200 Established，直接把之前截获的 Header 转发给目标
            target_stream.write_all(&initial_data_to_target).await?;
        }

        tokio::io::copy_bidirectional(&mut client, &mut target_stream).await?;
        return Ok(());
    }

    println!("Routing Engine: Cascade Tunnel -> {}:{}", host, port);

    // ==========================================
    // 2. 连接本地 VPN 端口
    // ==========================================
    let vpn_addr = format!("127.0.0.1:{}", config.vpn_port);
    
    // 增加 VPN 端口连通性校验
    let mut vpn_stream = match tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(&vpn_addr)).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            eprintln!("Error: Local VPN port {} connectivity check failed: {}", config.vpn_port, e);
            reply_error(&mut client, is_socks5, is_connect).await?;
            return Err(e.into());
        }
        Err(_) => {
            eprintln!("Error: Local VPN port {} connection timeout", config.vpn_port);
            reply_error(&mut client, is_socks5, is_connect).await?;
            return Err("VPN Connection timeout".into());
        }
    };

    // ==========================================
    // 3. 要求本地 VPN 建立通往远程 ISP 的隧道
    // ==========================================
    vpn_stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut vpn_resp = [0u8; 2];
    vpn_stream.read_exact(&mut vpn_resp).await?;
    if vpn_resp[0] != 0x05 || vpn_resp[1] != 0x00 {
        return Err("VPN Socks5 initial auth failed".into());
    }

    let ip_parts: Vec<u8> = config.isp_ip.split('.')
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
    // 4. 隧道打通后，在同一个 TCP 流上再次进行 Socks5 握手并鉴权 (发往 ISP)
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
    // 5. 将客户端最初请求的目标地址转发给远程 ISP
    // ==========================================
    vpn_stream.write_all(&dest_addr).await?;
    let mut isp_rep = [0u8; 4];
    vpn_stream.read_exact(&mut isp_rep).await?;
    if isp_rep[1] != 0x00 {
        return Err("Remote ISP failed to connect to target".into());
    }
    skip_socks5_addr(&mut vpn_stream, isp_rep[3]).await?;

    // ==========================================
    // 6. 告诉本地客户端通道已建立 (如果是 HTTPS 或 SOCKS)
    //    或者将读取的 Header 发往远程 ISP (如果是普通 HTTP)
    // ==========================================
    if is_socks5 {
        reply_success(&mut client, true).await?;
    } else if is_connect {
        reply_success(&mut client, false).await?;
    } else {
        vpn_stream.write_all(&initial_data_to_target).await?;
    }

    // ==========================================
    // 7. 使用 copy_bidirectional 双向转发流量
    // ==========================================
    tokio::io::copy_bidirectional(&mut client, &mut vpn_stream).await?;

    Ok(())
}

async fn reply_success(client: &mut TcpStream, is_socks5: bool) -> Result<(), std::io::Error> {
    if is_socks5 {
        client.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
    } else {
        client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
    }
    Ok(())
}

async fn reply_error(client: &mut TcpStream, is_socks5: bool, is_connect: bool) -> Result<(), std::io::Error> {
    if is_socks5 {
        // Socks5 0x05 (Connection refused)
        client.write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
    } else if is_connect {
        client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
    } else {
        client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await?;
    }
    Ok(())
}

/// 辅助函数：根据 ATYP 跳过 Socks5 返回的 Bind Address 信息
async fn skip_socks5_addr(stream: &mut TcpStream, atyp: u8) -> Result<(), Box<dyn Error + Send + Sync>> {
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