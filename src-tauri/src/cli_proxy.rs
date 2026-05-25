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

pub fn set_git_proxy(enable: bool, port: u16) -> Result<(), String> {
    let proxy_url = format!("http://127.0.0.1:{}", port);
    if enable {
        let _ = Command::new("git").args(["config", "--global", "http.proxy", &proxy_url]).output();
        let _ = Command::new("git").args(["config", "--global", "https.proxy", &proxy_url]).output();
        println!("Git proxy enabled");
    } else {
        let _ = Command::new("git").args(["config", "--global", "--unset", "http.proxy"]).output();
        let _ = Command::new("git").args(["config", "--global", "--unset", "https.proxy"]).output();
        println!("Git proxy disabled");
    }
    Ok(())
}

pub fn set_npm_proxy(enable: bool, port: u16) -> Result<(), String> {
    let proxy_url = format!("http://127.0.0.1:{}", port);
    let npm_cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
    
    if enable {
        let _ = Command::new(npm_cmd).args(["config", "set", "proxy", &proxy_url]).output();
        let _ = Command::new(npm_cmd).args(["config", "set", "https-proxy", &proxy_url]).output();
        println!("NPM proxy enabled");

        #[cfg(target_os = "windows")]
        {
            if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_SET_VALUE) {
                let _ = hkcu.set_value("NODE_TLS_REJECT_UNAUTHORIZED", &"0");
                broadcast_env_change();
            }
        }
    } else {
        let _ = Command::new(npm_cmd).args(["config", "delete", "proxy"]).output();
        let _ = Command::new(npm_cmd).args(["config", "delete", "https-proxy"]).output();
        println!("NPM proxy disabled");

        #[cfg(target_os = "windows")]
        {
            if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_SET_VALUE) {
                let _ = hkcu.delete_value("NODE_TLS_REJECT_UNAUTHORIZED");
                broadcast_env_change();
            }
        }
    }
    Ok(())
}