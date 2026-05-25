pub mod cascade_core;
use cascade_core::CascadeConfig;
use std::sync::Arc;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn start_cascade(
    vpn_port: u16,
    isp_ip: String,
    isp_port: u16,
    username: String,
    password: String,
) -> Result<String, String> {
    println!("Cascade Engine Initialized with dynamic parameters");
    
    let config = Arc::new(CascadeConfig {
        vpn_port,
        isp_ip,
        isp_port,
        username,
        password,
    });

    tokio::spawn(async move {
        if let Err(e) = cascade_core::start_server(config).await {
            eprintln!("Cascade Engine Fatal Error: {}", e);
        }
    });

    Ok("Cascade Engine started successfully".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_cascade])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
