use std::process::Command;

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
    } else {
        let _ = Command::new(npm_cmd).args(["config", "delete", "proxy"]).output();
        let _ = Command::new(npm_cmd).args(["config", "delete", "https-proxy"]).output();
        println!("NPM proxy disabled");
    }
    Ok(())
}