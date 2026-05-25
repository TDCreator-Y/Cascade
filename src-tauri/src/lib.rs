pub mod cascade_core;
pub mod sys_proxy;

use cascade_core::CascadeConfig;
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::task::JoinHandle;

struct AppState {
    server_handle: Mutex<Option<JoinHandle<()>>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn start_cascade(
    vpn_port: u16,
    isp_ip: String,
    isp_port: u16,
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    println!("Cascade Engine Initialized with dynamic parameters");
    
    let mut handle_lock = state.server_handle.lock().unwrap();
    if handle_lock.is_some() {
        return Err("Cascade Engine is already running".into());
    }

    // 开启系统全局代理
    if let Err(e) = sys_proxy::enable_sys_proxy(10808) {
        eprintln!("Failed to enable system proxy: {}", e);
        return Err(e);
    }

    let config = Arc::new(CascadeConfig {
        vpn_port,
        isp_ip,
        isp_port,
        username,
        password,
    });

    let handle = tokio::spawn(async move {
        if let Err(e) = cascade_core::start_server(config).await {
            eprintln!("Cascade Engine Fatal Error: {}", e);
        }
    });

    *handle_lock = Some(handle);

    Ok("Cascade Engine started and System Proxy taken over".to_string())
}

#[tauri::command]
async fn stop_cascade(state: State<'_, AppState>) -> Result<String, String> {
    let mut handle_lock = state.server_handle.lock().unwrap();
    if let Some(handle) = handle_lock.take() {
        handle.abort(); // 中止后台 TCP 监听任务
    }

    // 恢复系统代理
    if let Err(e) = sys_proxy::disable_sys_proxy() {
        eprintln!("Failed to disable system proxy: {}", e);
        return Err(e);
    }

    println!("Cascade Engine Stopped");
    Ok("Cascade Engine stopped and System Proxy restored".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(AppState {
            server_handle: Mutex::new(None),
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_cascade, stop_cascade])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
            // 确保应用退出时强制恢复系统代理
            println!("Application exiting, restoring system proxy...");
            let _ = sys_proxy::disable_sys_proxy();
        }
        _ => {}
    });
}
