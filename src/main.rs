fn main() {
    println!("Hostname: {}", read_host_name());
    println!("OS Name: {}", read_os_name());
    println!("Kernel: {}", read_kernel());
}

fn read_host_name() -> String {
    let hostname = std::fs::read_to_string("/etc/hostname");
    match hostname {
        Ok(contents) => contents.trim().to_string(),
        Err(_e) => "N/A".to_string(),
    }
}

fn read_os_name() -> String {
    let os_release = std::fs::read_to_string("/etc/os-release");
    match os_release {
        Err(_e) => "N/A".to_string(),
        Ok(contents) => {
            let mut lines = contents.split_terminator("\n");
            let os_pretty_name = lines.find(|&item| item.starts_with("PRETTY_NAME="));
            match os_pretty_name {
                Some(val) => val
                    .split("=")
                    .nth(1)
                    .expect("Can't read os name")
                    .trim()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
                    .trim()
                    .to_string(),
                None => "N/A".to_string(),
            }
        }
    }
}

fn read_kernel() -> String {
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease");
    match kernel {
        Err(_e) => "N/A".to_string(),
        Ok(contents) => contents.trim().to_string(),
    }
}
