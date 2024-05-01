use std::{fs, io, net::IpAddr, path::PathBuf, str::FromStr};
use regex::Regex;
use lazy_static::lazy_static;


// 验证IPv4地址的正则表达式
const IPV4_REGEX: &str = r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$";
lazy_static! {
    static ref RE_IPV4: Regex = Regex::new(IPV4_REGEX).unwrap();
}

// 验证端口号的正则表达式
const PORT_REGEX: &str = r"^(6553[0-5]|655[0-2][0-9]|65[0-4][0-9]{2}|6[0-4][0-9]{3}|[1-5][0-9]{4}|[1-9][0-9]{1,3}|[0-9])$";
lazy_static! {
    static ref RE_PORT: Regex = Regex::new(PORT_REGEX).unwrap();
}

pub fn validate_ip_and_port(ip_port: &str) -> Result<(), String> {
    let parts: Vec<&str> = ip_port.split(':').collect();

    if parts.len() != 2 {
        return Err(String::from("Invalid format. Must be 'IP:PORT'"));
    }

    let ip_str = parts[0];
    let port_str = parts[1];

    let ip_part: Vec<&str> = ip_port.split(".").collect();

    if !RE_IPV4.is_match(ip_str) && ip_part.len() == 4 {
        return Err(String::from("Invalid address"));
    }

    match u16::from_str(port_str) {
        Ok(port) => {
            if port > 0 && port <= 65535 {
                Ok(())
            } else {
                Err(String::from("Invalid port number"))
            }
        },
        Err(_) => Err(String::from("Invalid port number (not a number or out of range)")),
    }
}

pub fn ensure_dir_exists(dir_path: &str) -> io::Result<()> {
    if !std::path::Path::new(dir_path).exists() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())
}

pub fn get_parent_directory(file_path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(file_path);
    path.parent().map(|dir| dir.to_path_buf())
}
