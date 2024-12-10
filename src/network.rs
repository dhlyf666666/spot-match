use std::io;
use get_if_addrs::get_if_addrs;
use pnet::datalink;

pub fn get_ip_addresses() -> Result<Vec<String>, io::Error> {
    match get_if_addrs() {
        Ok(interface_vec) => {
            let ip_addresses: Vec<String> = interface_vec
                .into_iter()
                .map(|iface| iface.ip().to_string())
                .collect();
            Ok(ip_addresses)  // 将结果返回给外部
        }
        Err(e) => {
            Err(io::Error::new(io::ErrorKind::Other, format!("获取机器 IP 异常: {}", e)))
        }
    }
}

/// 获取系统中所有网络接口的 MAC 地址
pub fn get_mac_addresses() -> io::Result<Vec<String>> {
    let mut mac_addresses = Vec::new();
    for iface in datalink::interfaces() {
        if let Some(mac) = iface.mac {
            // 手动格式化 MAC 地址
            let mac_str = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                  mac.0, mac.1, mac.2, mac.3, mac.4, mac.5);
            mac_addresses.push(mac_str);
        }
    }
    Ok(mac_addresses)
}

/// 检查系统 MAC 地址是否包含指定的 `allowed_mac` 地址
pub fn check_mac_address(allowed_mac: &str) {
    match get_mac_addresses() {
        Ok(mac_addresses) => {
            if mac_addresses.contains(&allowed_mac.to_string()) {
                println!("MAC 地址符合要求，程序继续运行。");
            } else {
                panic!("MAC 地址不符，不允许运行！");
            }
        }
        Err(e) => {
            panic!("获取 MAC 地址异常: {}", e);
        }
    }
}
