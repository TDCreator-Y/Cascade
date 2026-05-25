use std::process::Command;

#[cfg(target_os = "windows")]
use winreg::{enums::*, RegKey};
#[cfg(target_os = "windows")]
use std::ptr::null_mut;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

#[cfg(target_os = "windows")]
fn broadcast_env_change() {
    unsafe {
        let lparam: Vec<u16> = "Environment\0".encode_utf16().collect();
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            lparam.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            null_mut(),
        );
    }
}

#[cfg(target_os = "windows")]
pub fn set_claude_proxy(enable: bool, port: u16) -> Result<(), String> {
    let proxy_url = format!("http://127.0.0.1:{}", port);
    
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_SET_VALUE) {
        if enable {
            let _ = hkcu.set_value("HTTP_PROXY", &proxy_url);
            let _ = hkcu.set_value("HTTPS_PROXY", &proxy_url);
            let _ = hkcu.set_value("NODE_TLS_REJECT_UNAUTHORIZED", &"0");
            println!("Claude (CLI) proxy enabled");
        } else {
            let _ = hkcu.delete_value("HTTP_PROXY");
            let _ = hkcu.delete_value("HTTPS_PROXY");
            let _ = hkcu.delete_value("NODE_TLS_REJECT_UNAUTHORIZED");
            println!("Claude (CLI) proxy disabled");
        }
        broadcast_env_change();
    } else {
        return Err("Failed to open registry key".to_string());
    }
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_claude_proxy(enable: bool, port: u16) -> Result<(), String> {
    // 暂不实现非 Windows 平台的逻辑
    Ok(())
}