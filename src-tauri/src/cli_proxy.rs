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

    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Environment", KEY_SET_VALUE)
    {
        if enable {
            let _ = hkcu.set_value("HTTP_PROXY", &proxy_url);
            let _ = hkcu.set_value("HTTPS_PROXY", &proxy_url);
            println!("Claude (CLI) proxy enabled");
        } else {
            let _ = hkcu.delete_value("HTTP_PROXY");
            let _ = hkcu.delete_value("HTTPS_PROXY");
            println!("Claude (CLI) proxy disabled");
        }
        broadcast_env_change();
    } else {
        return Err("Failed to open registry key".to_string());
    }

    Ok(())
}

// macOS：通过 launchctl 设置用户级环境变量
#[cfg(target_os = "macos")]
pub fn set_claude_proxy(enable: bool, port: u16) -> Result<(), String> {
    use std::process::Command;

    let vars = [
        ("HTTP_PROXY", format!("http://127.0.0.1:{}", port)),
        ("HTTPS_PROXY", format!("http://127.0.0.1:{}", port)),
    ];

    for (key, val) in &vars {
        let status = if enable {
            Command::new("launchctl")
                .args(["setenv", key, val])
                .status()
        } else {
            Command::new("launchctl")
                .args(["unsetenv", key])
                .status()
        };
        if let Err(e) = status {
            return Err(format!("launchctl failed for {}: {}", key, e));
        }
    }
    Ok(())
}

// Linux 及其他平台：暂不实现
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn set_claude_proxy(_enable: bool, _port: u16) -> Result<(), String> {
    Ok(())
}
