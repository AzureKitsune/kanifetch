use regex::Regex;
use which::which;

#[cfg(test)]
#[path = "hardware_tests.rs"]
mod tests;

#[cfg(not(test))]
fn fetch_cpu_info() -> Result<String, std::io::Error> {
    return std::fs::read_to_string("/proc/cpuinfo");
}

#[cfg(test)]
use tests::mock_fetch_cpu_info as fetch_cpu_info;

#[cfg(not(test))]
fn fetch_smt_info() -> Result<String, std::io::Error> {
    return std::fs::read_to_string("/sys/devices/system/cpu/smt/active");
}

#[cfg(test)]
use tests::mock_fetch_smt_info as fetch_smt_info;

#[cfg(not(test))]
fn fetch_cpu_boost() -> Result<String, std::io::Error> {
    return std::fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost");
}

#[cfg(test)]
use tests::mock_fetch_cpu_boost as fetch_cpu_boost;

#[allow(unused)]
#[derive(PartialEq, Eq, Debug)]
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
            "GenuineIntel" => CPUVendor::Intel,
            _ => CPUVendor::Unknown,
        }
    }

    pub fn to_str(self: &CPUVendor) -> &str {
        match self {
            CPUVendor::AMD => "AMD",
            CPUVendor::Intel => "Intel",
            _ => "",
        }
    }
}
#[allow(unused)]
#[derive(PartialEq, Eq, Debug)]
pub struct CPUInfo {
    pub cpu_name: String,
    pub core_count: i32,
    pub thread_count: i32,
    pub vendor: CPUVendor,
    pub is_smt_enabled: bool,
    pub is_turbo_boost_enabled: bool,
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

#[allow(unused)]
pub enum GPUVendor {
    AMD,
    Intel,
    Nvidia,
    Qualcomm,
    Unknown,
}

#[allow(unused)]
impl GPUVendor {
    pub fn from_str(value: &str) -> GPUVendor {
        match value {
            "AMD" => GPUVendor::AMD,
            "Intel" => GPUVendor::Intel,
            "Nvidia" => GPUVendor::Nvidia,
            _ => GPUVendor::Unknown,
        }
    }

    pub fn to_str(self: &GPUVendor) -> &str {
        match self {
            GPUVendor::AMD => "AMD",
            GPUVendor::Intel => "Intel",
            GPUVendor::Nvidia => "Nvidia",
            _ => "",
        }
    }
}

#[allow(unused)]
pub struct GPUInfo {
    pub gpu_name: String,
    pub gpu_vendor: GPUVendor,
    pub is_discrete: bool,
    pub vram: Option<String>,
}

#[allow(unused)]
pub struct MountInfo {
    pub mount_point: String,
    pub source: String,
    pub filesystem: String,
}

pub struct HardwareInfo {
    pub cpu_info: CPUInfo, // TODO account for multiple cpus
    pub gpus_info: Option<Vec<GPUInfo>>,
    pub memory_info: MemoryInfo,
    pub mount_info: Option<Vec<MountInfo>>,
}

pub fn get_cpu() -> CPUInfo {
    let cpu_info = fetch_cpu_info();

    let (cpu_name, core_count, thread_count, vendor) = match cpu_info {
        Err(_e) => ("N/A".to_string(), -1, -1, CPUVendor::Unknown),
        Ok(contents) => {
            let all_procs: Vec<_> = contents.split_terminator("\n\n").collect();

            let first_proc = all_procs.get(0);
            if first_proc.is_none() {
                ("N/A".to_string(), -1, -1, CPUVendor::Unknown);
            }

            let lines: Vec<_> = first_proc
                .unwrap()
                .split_terminator("\n")
                .map(|line| line.trim())
                .collect();
            let cpu_name_line = lines.iter().find(|&item| item.starts_with("model name"));
            let cpu_name = match cpu_name_line {
                Some(val) => val
                    .split(":")
                    .nth(1)
                    .expect("Can't read cpu name.")
                    .trim()
                    .to_string(),
                None => "N/A".to_string(),
            };

            let core_count_line = lines.iter().find(|&item| item.starts_with("cpu cores"));
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

            let thread_count_line = lines.iter().find(|&item| item.starts_with("siblings"));
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

            let vendor_line = lines.iter().find(|&item| item.starts_with("vendor_id"));
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
    let is_smt_enabled = match fetch_smt_info() {
        Err(_e) => false, // Assume false on error. TODO handle this better
        Ok(contents) => contents.trim() == "1",
    }; // this could be inferred from core_count == thread_count

    // https://www.kernel.org/doc/Documentation/cpu-freq/boost.txt
    let is_turbo_boost_enabled = match fetch_cpu_boost() {
        Err(_) => false, // Assume false
        Ok(contents) => contents.trim() == "1",
    };

    let mut formatted_cpu_name = cpu_name.clone();

    // TODO handle Intel, Nvidia and other CPUs
    match vendor {
        CPUVendor::AMD => {
            if cpu_name.contains("Radeon") {
                // Handle Ryzen iGPU

                let name_split = if cpu_name.contains("w/ Radeon") {
                    cpu_name.split("w/ Radeon").collect::<Vec<_>>()
                } else {
                    cpu_name.split("with Radeon").collect::<Vec<_>>()
                }; // Strips igpu name out

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
            } else {
                formatted_cpu_name = format!(
                    "{} ({} cores / {} threads)",
                    cpu_name, core_count, thread_count
                );
            }
        }
        CPUVendor::Intel => {
            formatted_cpu_name = cpu_name.replace("(R)", "").replace("(TM)", "");
        }
        CPUVendor::Nvidia => {}
        CPUVendor::Qualcomm => {}
        _ => {}
    }

    return CPUInfo {
        cpu_name: formatted_cpu_name,
        core_count,
        thread_count,
        vendor,
        is_smt_enabled,
        is_turbo_boost_enabled,
    };
}

pub fn get_hardware_info() -> HardwareInfo {
    let full_cpu_info = get_cpu();

    let all_gpus = get_gpus();

    let mem_info = get_memory().expect("Unable to gather memory information.");
    let mount_info = get_mounts();

    HardwareInfo {
        cpu_info: full_cpu_info,
        gpus_info: Some(all_gpus),
        memory_info: mem_info,
        mount_info,
    }
}

pub fn get_gpus() -> Vec<GPUInfo> {
    let mut detected_gpus = Vec::new();
    if std::path::Path::new("/proc/driver/nvidia/gpus/*/information").exists() {
        // handle Nvidia dGPUs

        let nvidia_device_re = Regex::new(r"(?mus)^Model\:.+?GPU Excluded\:.+?(Yes|No)").unwrap();

        let is_nvidia_smi_installed = which("nvidia-smi").is_ok();

        match std::fs::read_to_string("/proc/driver/nvidia/gpus/*/information") {
            Err(_e) => {}
            Ok(contents) => {
                for gpu_block in nvidia_device_re.find_iter(&contents) {
                    let mut gpu_block_lines = gpu_block.as_str().split_terminator("\n");

                    let model = gpu_block_lines
                        .find(|&item| item.starts_with("Model:"))
                        .unwrap();

                    let uuid = gpu_block_lines
                        .find(|&item| item.starts_with("GPU UUID:"))
                        .unwrap();

                    let vram = if is_nvidia_smi_installed {
                        let nvidia_smi_output = String::from_utf8(
                            std::process::Command::new("nvidia-smi")
                                .arg("-i")
                                .arg(uuid)
                                .arg("--query-gpu=memory.total")
                                .arg("--format=csv,noheader")
                                .output()
                                .expect("Unable to get nvidia-smi info.")
                                .stdout,
                        )
                        .expect("Unable to parse nvidia-smi info output.")
                        .replace(" ", "");

                        Some(nvidia_smi_output)
                    } else {
                        None
                    };

                    detected_gpus.push(GPUInfo {
                        gpu_name: model.to_string(),
                        gpu_vendor: GPUVendor::Nvidia,
                        is_discrete: true, // TODO handle future nvidia igpus
                        vram: vram,
                    });
                }
            }
        }
    }

    // detect AMD and Intel (d/i)GPUs via mesa. TODO handle llvmpipe (aka cpu/software rendering)
    if which("glxinfo").is_ok() {
        let glxinfo_output = String::from_utf8(
            std::process::Command::new("glxinfo")
                .arg("-B")
                .output()
                .expect("Unable to get opengl info.")
                .stdout,
        )
        .expect("Unable to parse opengl info output.");

        let device_name_info_re = Regex::new(r"(?mus)\(.+?\)").unwrap();

        let render_block_re = Regex::new(r"(?mus)^Extended renderer info.+?Memory info");
        let re_iter = render_block_re.unwrap();
        let matches = re_iter.find_iter(&glxinfo_output);

        for block_match in matches {
            let result = block_match.as_str();

            // TODO use regex capture groups

            let all_lines: Vec<_> = result.split_terminator("\n").collect();
            let mut all_lines_iter = all_lines.iter();

            let device_line = all_lines_iter
                .find(|&item| item.trim().starts_with("Device"))
                .expect("Unable to read device name.");
            let device_name = device_line
                .trim()
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse device name.")
                .trim();

            let video_mem_line = all_lines_iter
                .find(|&item| item.trim().starts_with("Video memory"))
                .expect("Unable to read video memory.");
            let video_mem = video_mem_line
                .trim()
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse video memory.")
                .trim()
                .split(" ")
                .nth(0)
                .unwrap_or("N/A")
                .trim();

            let unified_mem_line = all_lines_iter
                .find(|&item| item.trim().starts_with("Unified memory"))
                .expect("Unable to read unified memory.");
            let unified_mem = unified_mem_line
                .trim()
                .split(":")
                .collect::<Vec<_>>()
                .get(1)
                .expect("Unable to parse unified memory.")
                .trim()
                .split(" ")
                .nth(0)
                .unwrap_or("N/A")
                .trim();

            let is_discrete = match unified_mem {
                // Assume unified memory = integrated gpu
                "yes" => false,
                _ => true,
            };

            let vendor = if device_name.contains("AMD") || device_name.contains("ATI") {
                GPUVendor::AMD
            } else if device_name.contains("Intel") {
                GPUVendor::Intel
            } else {
                GPUVendor::Unknown
            };

            let formatted_device_name = match vendor {
                GPUVendor::AMD => device_name_info_re
                    .replace_all(device_name, "")
                    .trim()
                    .to_string(),
                GPUVendor::Intel => device_name.replace("(R)", ""),
                _ => device_name.to_string(),
            };

            detected_gpus.push(GPUInfo {
                gpu_name: formatted_device_name.to_string(),
                gpu_vendor: vendor,
                is_discrete: is_discrete,
                vram: Some(video_mem.to_string()),
            });
        }
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

pub fn get_mounts() -> Option<Vec<MountInfo>> {
    let wrapped_mounts = std::fs::read_to_string("/proc/mounts");
    match wrapped_mounts {
        Err(_) => None,
        Ok(contents) => {
            let mut detected_mounts: Vec<_> = Vec::new();
            let mut added_sources: Vec<String> = Vec::new();
            let all_mounts: Vec<_> = contents.split_terminator("\n").collect();
            for mount in all_mounts {
                let mount_parts = mount.split(' ').collect::<Vec<&str>>();
                let source = mount_parts.get(0).unwrap();
                let mount_point = mount_parts.get(1).unwrap();
                let filesystem = mount_parts.get(2).unwrap();
                if source.starts_with("/dev") && !added_sources.contains(&source.to_string()) {
                    detected_mounts.push(MountInfo {
                        mount_point: mount_point.to_string(),
                        source: source.to_string(),
                        filesystem: filesystem.to_string(),
                    });
                    added_sources.push(source.to_string());
                }
            }

            Some(detected_mounts)
        }
    }
}
