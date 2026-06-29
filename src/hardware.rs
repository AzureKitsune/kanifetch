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
            "Intel" => CPUVendor::Intel,
            _ => CPUVendor::Unknown,
        }
    }
}

#[allow(unused)]
pub struct CPUInfo {
    pub cpu_name: String,
    pub core_count: i32,
    pub thread_count: i32,
    pub vendor: CPUVendor,
    pub is_smt_enabled: bool,
    pub is_turbo_boost_enabled: bool,
    pub igpu_name: Option<String>,
}

#[allow(unused)]
pub struct MemoryInfo {
    pub total_mem: usize,
    // pub free_mem: usize,
    pub available_mem: usize,
    pub used_mem: usize,

    pub total_swap: usize,
    pub used_swap: usize,
}

pub struct HardwareInfo {
    pub cpu_info: CPUInfo, // TODO account for multiple cpus
    pub discrete_gpu_names: Option<Vec<String>>,
    pub memory_info: MemoryInfo,
}

pub fn get_cpu() -> CPUInfo {
    let cpu_info = std::fs::read_to_string("/proc/cpuinfo");
    let mut igpu_name: Option<String> = None;
    let mut formatted_cpu_name = String::new();
    let (cpu_name, core_count, thread_count, vendor) = match cpu_info {
        Err(_e) => ("N/A".to_string(), -1, -1, CPUVendor::Unknown),
        Ok(contents) => {
            let all_procs: Vec<_> = contents.split_terminator("\n\n").collect();

            let first_proc = all_procs.get(0);
            if first_proc.is_none() {
                ("N/A".to_string(), -1, -1, CPUVendor::Unknown);
            }

            let mut lines = first_proc.unwrap().split_terminator("\n");
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

    // https://www.kernel.org/doc/Documentation/cpu-freq/boost.txt
    let is_turbo_boost_enabled =
        match std::fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost") {
            Err(_) => false, // Assume false
            Ok(contents) => contents.trim() == "1",
        };

    let mut formatted_cpu_name = cpu_name.clone();

    // TODO handle Intel, Nvidia and other CPUs
    match vendor {
        CPUVendor::AMD => {
            if cpu_name.contains("w/ Radeon") {
                // Ryzen iGPU

                let name_split = cpu_name.split("w/ Radeon").collect::<Vec<_>>();

                formatted_cpu_name = format!(
                    "{} ({} cores / {} threads)",
                    name_split
                        .get(0)
                        .expect("Unable to parse cpu name.")
                        .trim()
                        .to_string(),
                    core_count,
                    thread_count
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
                formatted_cpu_name = format!(
                    "{} ({} cores / {} threads)",
                    cpu_name, core_count, thread_count
                );
            }
        }
        CPUVendor::Intel => {}
        CPUVendor::Nvidia => {}
        CPUVendor::Qualcomm => {}
        _ => {}
    }

    return CPUInfo {
        cpu_name: formatted_cpu_name,
        core_count,
        thread_count,
        vendor,
        igpu_name,
        is_smt_enabled,
        is_turbo_boost_enabled,
    };
}

pub fn get_hardware_info() -> HardwareInfo {
    let full_cpu_info = get_cpu();

    let mut discrete_gpus = get_discrete_gpus();
    if full_cpu_info.igpu_name != None {
        // Removes igpu from discrete gpu list
        let full_igpu_name = discrete_gpus
            .iter()
            .find(|&item| {
                item.contains(
                    full_cpu_info
                        .igpu_name
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

    let mem_info = get_memory().expect("Unable to gather memory information.");

    HardwareInfo {
        cpu_info: full_cpu_info,
        discrete_gpu_names: Some(discrete_gpus),
        memory_info: mem_info,
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
    let glxinfo_output = String::from_utf8(
        std::process::Command::new("glxinfo")
            .output()
            .expect("Unable to get opengl info.")
            .stdout,
    )
    .expect("Unable to parse opengl info output.");
    let opengl_renders = glxinfo_output
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

pub fn get_memory() -> Result<MemoryInfo, &'static str> {
    let mem_info = std::fs::read_to_string("/proc/meminfo");

    match mem_info {
        Err(_) => Err("Unable to retrieve memory information."),
        Ok(contents) => {
            let all_lines: Vec<_> = contents.split_terminator("\n").collect();
            let mut all_lines_iter = all_lines.iter();

            let total_mem_line = all_lines_iter
                .find(|&item| item.starts_with("MemTotal"))
                .expect("Unable to read total memory.");
            let total_mem = total_mem_line
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse total memory.")
                .trim()
                .split(" ")
                .nth(0)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);

            let available_mem_line = all_lines_iter
                .find(|&item| item.starts_with("MemAvailable"))
                .expect("Unable to read available memory.");
            let available_mem = available_mem_line
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse available memory.")
                .trim()
                .split(" ")
                .nth(0)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);

            let total_swap_line = all_lines_iter
                .find(|&item| item.starts_with("SwapTotal"))
                .expect("Unable to read total swap memory.");
            let total_swap = total_swap_line
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse total swap memory.")
                .trim()
                .split(" ")
                .nth(0)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);

            let swap_free_line = all_lines_iter
                .find(|&item| item.starts_with("SwapFree"))
                .expect("Unable to read free swap memory.");
            let swap_free = swap_free_line
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse free swap memory.")
                .trim()
                .split(" ")
                .nth(0)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);

            Ok(MemoryInfo {
                total_mem: total_mem as usize,
                available_mem: available_mem as usize,
                used_mem: (total_mem - available_mem) as usize,
                total_swap: total_swap as usize,
                used_swap: (total_swap - swap_free) as usize,
            })
        }
    }
}
