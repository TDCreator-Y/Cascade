pub mod cascade_core;
pub mod sys_proxy;
pub mod cli_proxy;

use cascade_core::CascadeConfig;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};
use tokio::task::JoinHandle;

struct AppState {
    server_handle: Mutex<Option<JoinHandle<()>>>,
}

#[tauri::command]
fn toggle_claude_proxy(enable: bool) -> Result<(), String> {
    cli_proxy::set_claude_proxy(enable, 10808)
}

#[tauri::command]
async fn start_cascade(
    vpn_port: u16,
    isp_ip: String,
    isp_port: u16,
    username: String,
    password: String,
    claude_proxy_enabled: bool,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 持锁检查后立即释放，不跨 await 持锁
    {
        let handle_lock = state.server_handle.lock().unwrap();
        if handle_lock.is_some() {
            return Err("Cascade Engine 已在运行".into());
        }
    }

    // 预绑定端口：若端口被占用立即返回友好错误（await 在锁外）
    let listener = tokio::net::TcpListener::bind("127.0.0.1:10808")
        .await
        .map_err(|e| {
            format!(
                "启动失败：端口 10808 已被占用，请检查是否有其他程序使用该端口 ({})",
                e
            )
        })?;

    // VPN 端口连通性预检（await 在锁外）
    match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", vpn_port)),
    )
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            return Err(format!(
                "VPN 端口 {} 无法连接，请确认 VPN 客户端已启动并监听该端口 ({})",
                vpn_port, e
            ));
        }
        Err(_) => {
            return Err(format!(
                "VPN 端口 {} 连接超时，请确认 VPN 客户端已启动",
                vpn_port
            ));
        }
    }

    if let Err(e) = sys_proxy::enable_sys_proxy(10808) {
        eprintln!("Failed to enable system proxy: {}", e);
        return Err(e);
    }

    let _ = cli_proxy::set_claude_proxy(claude_proxy_enabled, 10808);

    let config = Arc::new(CascadeConfig {
        vpn_port,
        isp_ip,
        isp_port,
        username,
        password,
    });

    // 日志通道：cascade_core → mpsc → Tauri event
    let (log_tx, mut log_rx) = tokio::sync::mpsc::channel::<String>(256);
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(msg) = log_rx.recv().await {
            let _ = app_clone.emit("cascade-log", msg);
        }
    });

    let handle = tokio::spawn(async move {
        if let Err(e) = cascade_core::start_server(listener, config, log_tx).await {
            eprintln!("Cascade Engine Fatal Error: {}", e);
        }
    });

    // 再次加锁写入 handle（检查期间不应有第二个启动，但防御性双检）
    {
        let mut handle_lock = state.server_handle.lock().unwrap();
        if handle_lock.is_some() {
            handle.abort();
            return Err("Cascade Engine 已在运行（并发启动冲突）".into());
        }
        *handle_lock = Some(handle);
    }

    Ok("Cascade Engine 已启动，系统代理已接管".to_string())
}

#[tauri::command]
async fn stop_cascade(state: State<'_, AppState>) -> Result<String, String> {
    // 取出 handle 后立即释放锁，abort 在锁外执行
    let handle = {
        let mut handle_lock = state.server_handle.lock().unwrap();
        handle_lock.take()
    };

    if let Some(h) = handle {
        h.abort();
    }

    if let Err(e) = sys_proxy::disable_sys_proxy() {
        eprintln!("Failed to disable system proxy: {}", e);
        return Err(e);
    }

    let _ = cli_proxy::set_claude_proxy(false, 10808);

    println!("Cascade Engine Stopped");
    Ok("Cascade Engine 已停止，系统代理已恢复".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(AppState {
            server_handle: Mutex::new(None),
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_cascade,
            stop_cascade,
            toggle_claude_proxy
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
            println!("Application exiting, restoring system proxy...");
            let _ = sys_proxy::disable_sys_proxy();
            let _ = cli_proxy::set_claude_proxy(false, 10808);
        }
        _ => {}
    });
}
