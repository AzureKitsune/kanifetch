use std::fmt::format;

pub enum CPUVendor {
    AMD,
    Intel,
    Nvidia,
    Qualcomm,
    Unknown,
}

impl CPUVendor {
    pub fn from_str(value: &str) -> CPUVendor {
        match value {
            "AuthenticAMD" => CPUVendor::AMD,
            _ => CPUVendor::Unknown,
        }
    }
}

pub struct CPUInfo {
    pub cpu_name: String,
    pub core_count: i32,
    pub vendor: CPUVendor,
}

pub struct HardwareInfo {
    pub cpu_name: String, // TODO account for multiple cpus
    pub integrated_gpu_name: Option<String>,
    pub discrete_gpu_names: Option<Vec<String>>,
}

pub fn get_cpu() -> CPUInfo {
    let cpu_info = std::fs::read_to_string("/proc/cpuinfo");
    match cpu_info {
        Err(_e) => {
            return CPUInfo {
                cpu_name: "N/A".to_string(),
                core_count: -1,
                vendor: CPUVendor::Unknown,
            };
        }
        Ok(contents) => {
            let mut lines = contents.split_terminator("\n");
            let cpu_name_line = lines.find(|&item| item.starts_with("model name"));
            let cpu_name = match cpu_name_line {
                Some(val) => val
                    .split(":")
                    .nth(1)
                    .expect("Can't read cpu name")
                    .trim()
                    .to_string(),
                None => "N/A".to_string(),
            };

            //TODO maybe iterate only the first core for all the info we need (model name, core count, etc)
            let core_count_line = lines.find(|&item| item.starts_with("cpu cores"));
            let core_count = match core_count_line {
                Some(val) => val
                    .split(":")
                    .nth(1)
                    .expect("Can't read cpu core count")
                    .trim()
                    .parse()
                    .expect("Can't parse cpu core count"),
                None => -1,
            };

            let vendor_line = lines.find(|&item| item.starts_with("vendor_id"));
            let vendor = match vendor_line {
                Some(val) => CPUVendor::from_str(
                    val.split(":")
                        .nth(1)
                        .expect("Can't read vendor name")
                        .trim(),
                ),
                None => CPUVendor::Unknown,
            };

            return CPUInfo {
                cpu_name,
                core_count,
                vendor,
            };
        }
    }
}

pub fn get_hardware_info() -> HardwareInfo {
    let full_cpu_info = get_cpu();
    let mut cpu_name = String::from(full_cpu_info.cpu_name.clone());
    let mut igpu_name: Option<String> = None;

    match full_cpu_info.vendor {
        CPUVendor::AMD => {
            if full_cpu_info.cpu_name.contains("w/ Radeon") {
                // Ryzen iGPU

                let name_split = full_cpu_info
                    .cpu_name
                    .split("w/ Radeon")
                    .collect::<Vec<_>>();

                cpu_name = format!(
                    "{} ({} cores)",
                    name_split
                        .get(0)
                        .expect("Unable to parse cpu name")
                        .trim()
                        .to_string(),
                    full_cpu_info.core_count
                );
                igpu_name = Some(format!(
                    "AMD Radeon {}",
                    name_split
                        .get(1)
                        .expect("Unable to parse igpu name")
                        .trim()
                        .to_string()
                ));
            } else {
                cpu_name = format!(
                    "{} ({} cores)",
                    full_cpu_info.cpu_name, full_cpu_info.core_count
                );
            }
        }
        CPUVendor::Intel => {}
        CPUVendor::Nvidia => {}
        CPUVendor::Qualcomm => {}
        _ => {}
    }

    //TODO handle Intel, Nvidia and other CPUs

    HardwareInfo {
        cpu_name,
        integrated_gpu_name: igpu_name,
        discrete_gpu_names: None,
    }
}
