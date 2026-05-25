pub mod cascade_core;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn start_cascade() -> Result<String, String> {
    println!("Cascade Engine Initialized");
    
    tokio::spawn(async {
        if let Err(e) = cascade_core::start_server().await {
            eprintln!("Cascade Engine Fatal Error: {}", e);
        }
    });

    Ok("Cascade Engine started successfully".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_cascade])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}