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
    println!("Cascade Engine: Socks5 listening on 127.0.0.1:10808");

    loop {
        let (client, addr) = listener.accept().await?;
        println!("Cascade Engine: Accepted connection from {}", addr);
        
        let cfg = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = handle_client(client, cfg).await {
                eprintln!("Cascade Engine Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut client: TcpStream, config: Arc<CascadeConfig>) -> Result<(), Box<dyn Error + Send + Sync>> {
    // ==========================================
    // 1. 本地 Socks5 握手阶段 (处理来自本地软件的请求)
    // ==========================================
    let mut buf = [0u8; 2];
    client.read_exact(&mut buf).await?;
    if buf[0] != 0x05 {
        return Err("Invalid Socks5 version from local client".into());
    }

    let nmethods = buf[1] as usize;
    let mut methods = vec![0u8; nmethods];
    client.read_exact(&mut methods).await?;

    // 回复无须认证 (No Auth)
    client.write_all(&[0x05, 0x00]).await?;

    // 读取连接目标请求
    let mut req_header = [0u8; 4];
    client.read_exact(&mut req_header).await?;
    if req_header[1] != 0x01 {
        return Err("Only CONNECT command is supported".into());
    }

    let atyp = req_header[3];
    let mut dest_addr = Vec::new();
    dest_addr.extend_from_slice(&req_header);

    // 解析目标地址
    match atyp {
        0x01 => {
            // IPv4
            let mut addr = [0u8; 6]; // 4 字节 IP + 2 字节端口
            client.read_exact(&mut addr).await?;
            dest_addr.extend_from_slice(&addr);
        }
        0x03 => {
            // 域名
            let mut len = [0u8; 1];
            client.read_exact(&mut len).await?;
            dest_addr.push(len[0]);
            let mut addr = vec![0u8; len[0] as usize + 2]; // 域名内容 + 2 字节端口
            client.read_exact(&mut addr).await?;
            dest_addr.extend_from_slice(&addr);
        }
        0x04 => {
            // IPv6
            let mut addr = [0u8; 18]; // 16 字节 IP + 2 字节端口
            client.read_exact(&mut addr).await?;
            dest_addr.extend_from_slice(&addr);
        }
        _ => return Err("Unsupported address type".into()),
    }

    // ==========================================
    // 2. 连接本地 VPN 端口
    // ==========================================
    let vpn_addr = format!("127.0.0.1:{}", config.vpn_port);
    let mut vpn_stream = TcpStream::connect(&vpn_addr).await?;

    // ==========================================
    // 3. 要求本地 VPN 建立通往远程 ISP 的隧道
    // ==========================================
    // 与 VPN 握手
    vpn_stream.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut vpn_resp = [0u8; 2];
    vpn_stream.read_exact(&mut vpn_resp).await?;
    if vpn_resp[0] != 0x05 || vpn_resp[1] != 0x00 {
        return Err("VPN Socks5 initial auth failed".into());
    }

    // 解析 isp_ip 为 IPv4 字节
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
    // 跳过 Bind Address 数据
    skip_socks5_addr(&mut vpn_stream, vpn_rep[3]).await?;

    // ==========================================
    // 4. 隧道打通后，在同一个 TCP 流上再次进行 Socks5 握手并鉴权 (发往 ISP)
    // ==========================================
    // 发送支持 0x02(Username/Password) 认证方式
    vpn_stream.write_all(&[0x05, 0x01, 0x02]).await?;
    let mut isp_resp = [0u8; 2];
    vpn_stream.read_exact(&mut isp_resp).await?;
    if isp_resp[1] != 0x02 {
        return Err("Remote ISP did not accept Auth method 0x02".into());
    }

    // 发送鉴权信息 (需替换为实际的用户名密码)
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
    // 6. 告诉本地客户端通道已建立，准备转发数据
    // ==========================================
    // 返回全 0 地址作为成功响应
    client
        .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .await?;

    // ==========================================
    // 7. 使用 copy_bidirectional 双向转发流量
    // ==========================================
    tokio::io::copy_bidirectional(&mut client, &mut vpn_stream).await?;

    Ok(())
}

/// 辅助函数：根据 ATYP 跳过 Socks5 返回的 Bind Address 信息
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
