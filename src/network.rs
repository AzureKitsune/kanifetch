pub struct NetworkInterfaceInfo {
    pub name: String,
    pub address: String,
    pub address_type: String,
}

pub fn get_ipv4_info() -> Result<Vec<NetworkInterfaceInfo>, &'static str> {
    let wrapped_ip_output = std::process::Command::new("ip")
        .arg("--oneline")
        .arg("address")
        .arg("show")
        .output();

    match wrapped_ip_output {
        Err(_) => Err("Unable to get network information."),
        Ok(ip_output) => {
            let network_info = String::from_utf8(ip_output.stdout)
                .expect("Unable to decode network information.")
                .trim()
                .to_string();

            let mut ip_list: Vec<NetworkInterfaceInfo> = Vec::new();

            for line in network_info.split_terminator("\n") {
                if line.contains("inet") && !line.contains("inet6") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    let ifname = fields
                        .get(1)
                        .expect("Unable to get network interface name.")
                        .to_string();
                    let protocol = fields
                        .get(2)
                        .expect("Unable to get network interface protocol.")
                        .to_string();

                    let ip_addr = fields
                        .get(3)
                        .expect("Unable to get network interface address.")
                        .to_string();

                    if ifname == "lo" || protocol != "inet" {
                        continue;
                    }

                    let result = NetworkInterfaceInfo {
                        name: ifname,
                        address: ip_addr,
                        address_type: protocol,
                    };

                    ip_list.push(result);
                }
            }

            return Ok(ip_list);
        }
    }
}
