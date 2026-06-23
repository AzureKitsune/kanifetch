#[allow(unused)]
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
    pub thread_count: i32,
    pub vendor: CPUVendor,
    pub is_smt_enabled: bool,
}

pub struct HardwareInfo {
    pub cpu_name: String, // TODO account for multiple cpus
    pub is_cpu_smt_enabled: bool,
    pub integrated_gpu_name: Option<String>,
    pub discrete_gpu_names: Option<Vec<String>>,
}

pub fn get_cpu() -> CPUInfo {
    let cpu_info = std::fs::read_to_string("/proc/cpuinfo");
    let (cpu_name, core_count, thread_count, vendor) = match cpu_info {
        Err(_e) => ("N/A".to_string(), -1, -1, CPUVendor::Unknown),
        Ok(contents) => {
            let mut lines = contents.split_terminator("\n");
            let cpu_name_line = lines.find(|&item| item.starts_with("model name"));
            let cpu_name = match cpu_name_line {
                Some(val) => val
                    .split(":")
                    .nth(1)
                    .expect("Can't read cpu name.")
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
                    .expect("Can't read cpu core count.")
                    .trim()
                    .parse()
                    .expect("Can't parse cpu core count."),
                None => -1,
            };

            let thread_count_line = lines.find(|&item| item.starts_with("siblings"));
            let thread_count = match thread_count_line {
                Some(val) => val
                    .split(":")
                    .nth(1)
                    .expect("Can't read cpu thread count.")
                    .trim()
                    .parse()
                    .expect("Can't parse cpu thread count."),
                None => -1,
            };

            let vendor_line = lines.find(|&item| item.starts_with("vendor_id"));
            let vendor = match vendor_line {
                Some(val) => CPUVendor::from_str(
                    val.split(":")
                        .nth(1)
                        .expect("Can't read vendor name.")
                        .trim(),
                ),
                None => CPUVendor::Unknown,
            };

            (cpu_name, core_count, thread_count, vendor)
        }
    };

    // https://unix.stackexchange.com/a/522295
    let is_smt_enabled = match std::fs::read_to_string("/sys/devices/system/cpu/smt/active") {
        Err(_e) => false, // Assume false on error. TODO handle this better
        Ok(contents) => contents.trim() == "1",
    }; // this could be inferred from core_count == thread_count

    return CPUInfo {
        cpu_name,
        core_count,
        thread_count,
        vendor,
        is_smt_enabled,
    };
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
                    "{} ({} cores / {} threads)",
                    name_split
                        .get(0)
                        .expect("Unable to parse cpu name.")
                        .trim()
                        .to_string(),
                    full_cpu_info.core_count,
                    full_cpu_info.thread_count
                );
                igpu_name = Some(format!(
                    "AMD Radeon {}",
                    name_split
                        .get(1)
                        .expect("Unable to parse igpu name.")
                        .trim()
                        .to_string()
                ));
            } else {
                cpu_name = format!(
                    "{} ({} cores / {} threads)",
                    full_cpu_info.cpu_name, full_cpu_info.core_count, full_cpu_info.thread_count
                );
            }
        }
        CPUVendor::Intel => {}
        CPUVendor::Nvidia => {}
        CPUVendor::Qualcomm => {}
        _ => {}
    }

    // TODO handle Intel, Nvidia and other CPUs

    let mut discrete_gpus = get_discrete_gpus();
    if igpu_name != None {
        // Removes igpu from discrete gpu list
        let full_igpu_name = discrete_gpus
            .iter()
            .find(|&item| {
                item.contains(
                    igpu_name
                        .clone()
                        .expect("Unable to get igpu name.")
                        .as_str(),
                )
            })
            .expect("Unable to get igpu name.");
        discrete_gpus = discrete_gpus
            .iter()
            .filter(|&item| item != full_igpu_name)
            .map(|item| item.to_string())
            .collect();
    }

    HardwareInfo {
        cpu_name,
        is_cpu_smt_enabled: full_cpu_info.is_smt_enabled,
        integrated_gpu_name: igpu_name,
        discrete_gpu_names: Some(discrete_gpus),
    }
}

pub fn get_discrete_gpus() -> Vec<String> {
    let mut detected_gpus = Vec::new();
    if std::path::Path::new("/proc/driver/nvidia/gpus/*/information").exists() {
        // handle Nvidia dGPUs
        match std::fs::read_to_string("/proc/driver/nvidia/gpus/*/information") {
            Err(_e) => {}
            Ok(contents) => {
                let model_lines = contents
                    .split_terminator("\n")
                    .filter(|&item| item.starts_with("Model:"));

                for line in model_lines {
                    let gpu_name: String = line
                        .split(":")
                        .collect::<Vec<_>>()
                        .get(1)
                        .expect("Unable to fetch gpu name.")
                        .trim()
                        .to_string();

                    detected_gpus.push(gpu_name);
                }
            }
        }
    }

    // detect AMD and Intel dGPUs via mesa
    let glinfo_output = String::from_utf8(
        std::process::Command::new("glxinfo")
            .output()
            .expect("Unable to get opengl info.")
            .stdout,
    )
    .expect("Unable to parse opengl info output.");
    let opengl_renders = glinfo_output
        .split_terminator('\n')
        .filter(|&item| item.starts_with("OpenGL renderer string:") && !item.contains("Nvidia"))
        .map(|item| {
            item.split(":")
                .last()
                .expect("Unable to parse opengl render info.")
                .trim()
        });

    for gpu in opengl_renders {
        detected_gpus.push(gpu.to_string());
    }

    return detected_gpus;
}
