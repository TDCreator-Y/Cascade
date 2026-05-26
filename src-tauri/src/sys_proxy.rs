// Windows 实现：通过注册表控制 WinINet 代理
#[cfg(target_os = "windows")]
use windows_sys::Win32::Networking::WinInet::{
    InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED,
};
#[cfg(target_os = "windows")]
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
pub fn enable_sys_proxy(port: u16) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";
    let key = hkcu
        .open_subkey_with_flags(path, KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    key.set_value("ProxyEnable", &1u32)
        .map_err(|e| format!("Failed to set ProxyEnable: {}", e))?;
    let proxy_server = format!("127.0.0.1:{}", port);
    key.set_value("ProxyServer", &proxy_server)
        .map_err(|e| format!("Failed to set ProxyServer: {}", e))?;

    refresh_system_proxy();
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn disable_sys_proxy() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";
    let key = hkcu
        .open_subkey_with_flags(path, KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    key.set_value("ProxyEnable", &0u32)
        .map_err(|e| format!("Failed to set ProxyEnable: {}", e))?;

    refresh_system_proxy();
    Ok(())
}

#[cfg(target_os = "windows")]
fn refresh_system_proxy() {
    unsafe {
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_SETTINGS_CHANGED,
            std::ptr::null(),
            0,
        );
        InternetSetOptionW(
            std::ptr::null_mut(),
            INTERNET_OPTION_REFRESH,
            std::ptr::null(),
            0,
        );
    }
}

// macOS 实现：通过 networksetup 命令行工具设置系统代理
#[cfg(target_os = "macos")]
pub fn enable_sys_proxy(port: u16) -> Result<(), String> {
    use std::process::Command;

    let port_str = port.to_string();
    for service in get_macos_network_services() {
        let _ = Command::new("networksetup")
            .args(["-setwebproxy", &service, "127.0.0.1", &port_str])
            .status();
        let _ = Command::new("networksetup")
            .args(["-setsecurewebproxy", &service, "127.0.0.1", &port_str])
            .status();
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn disable_sys_proxy() -> Result<(), String> {
    use std::process::Command;

    for service in get_macos_network_services() {
        let _ = Command::new("networksetup")
            .args(["-setwebproxystate", &service, "off"])
            .status();
        let _ = Command::new("networksetup")
            .args(["-setsecurewebproxystate", &service, "off"])
            .status();
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn get_macos_network_services() -> Vec<String> {
    use std::process::Command;

    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .unwrap_or_default();
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .skip(1) // 第一行是说明文字
        .filter(|l| !l.starts_with('*'))
        .map(|l| l.to_string())
        .collect()
}

// Linux 及其他平台：暂不实现
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn enable_sys_proxy(_port: u16) -> Result<(), String> {
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn disable_sys_proxy() -> Result<(), String> {
    Ok(())
}
