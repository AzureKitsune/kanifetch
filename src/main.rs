use std::fs;

fn main() {
    let hostname: String = read_host_name();
    println!("Hostname: {}", hostname);
}

fn read_host_name() -> String {
    let hostname = std::fs::read_to_string("/etc/hostname");
    match hostname {
        Ok(contents) => contents,
        Err(e) => "N/A".to_string(),
    }
}
