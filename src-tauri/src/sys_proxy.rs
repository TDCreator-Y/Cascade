use windows_sys::Win32::Networking::WinInet::{
    InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED,
};
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
use winreg::RegKey;

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