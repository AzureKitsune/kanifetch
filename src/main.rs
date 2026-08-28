mod colors;
mod desktop_env;
mod hardware;
mod network;
mod pkg;
mod shell;

use colors::TerminalColors;
use shell::get_shell;

use crate::{
    hardware::{CPUInfo, GPUInfo, MemoryInfo, NPUInfo, get_hardware_info},
    network::NetworkInterfaceInfo,
};

fn main() {
    let os_name = read_os_name();
    let kernel = read_kernel();
    let uptime = read_uptime();
    let shell_name = get_shell();
    let terminal = detect_terminal();
    let wrapped_pkg_info = pkg::get_installed_system_pkg();
    let wrapped_flatpak_info = pkg::get_installed_flatpaks();
    let wrapped_snap_info = pkg::get_installed_snaps();
    let desktop_env = desktop_env::get_desktop_env();
    let net_info = network::get_ipv4_info();
    let hardware_info = get_hardware_info();

    print_header();

    colors::print_color(&TerminalColors::BoldGreen, "OS: ");
    colors::println_color(&TerminalColors::White, os_name);

    colors::print_color(&TerminalColors::BoldRed, "Kernel: ");
    colors::println_color(&TerminalColors::White, kernel);

    colors::print_color(&TerminalColors::BoldCyan, "Uptime: ");
    colors::println_color(&TerminalColors::White, uptime);

    colors::print_color(&TerminalColors::BoldBlue, "Shell: ");
    colors::println_color(&TerminalColors::White, shell_name);

    colors::print_color(&TerminalColors::BoldRed, "Terminal: ");
    colors::println_color(&TerminalColors::White, terminal);

    print_package_info(&wrapped_pkg_info, &wrapped_flatpak_info, &wrapped_snap_info);

    colors::print_color(&TerminalColors::BoldGreen, "DE: ");
    colors::println_color(
        &TerminalColors::White,
        format!(
            "{} ({})",
            desktop_env.desktop_env, desktop_env.desktop_env_version
        ),
    );
    colors::print_color(&TerminalColors::BoldCyan, "WM: ");
    let colorized_session_type = match desktop_env.session_type.as_str() {
        "wayland" => colors::wrap_color(&TerminalColors::BoldBlue, format!("[{}]", "Wayland")),
        "x11" => colors::wrap_color(&TerminalColors::BoldGreen, format!("[{}]", "X11")),
        "tty" => colors::wrap_color(&TerminalColors::BoldPurple, format!("[{}]", "TTY")),
        _ => format!("[{}]", desktop_env.session_type.as_str().to_string()),
    };
    colors::println_color(
        &TerminalColors::White,
        format!(
            "{} ({}) {}",
            desktop_env.window_manager, desktop_env.window_manager_version, colorized_session_type
        ),
    );

    if let Ok(unwrapped_net_info) = net_info {
        print_network_info(unwrapped_net_info);
    }

    print_cpu_info(&hardware_info.cpu_info);

    print_gpu_info(&hardware_info.gpus_info);

    if let Some(npu_info) = &hardware_info.npu_info {
        print_npu_info(&npu_info);
    }

    print_memory_info(hardware_info.memory_info);

    if let Some(mount_info) = &hardware_info.mount_info {
        for mount in mount_info {
            colors::print_color(&TerminalColors::BoldRed, "Disk: ");
            colors::println_color(&TerminalColors::White, mount.mount_point.clone());
        }
    }
}

fn format_memory_amount(memory: usize) -> String {
    if memory >= 1048576 {
        // Gibibytes
        return format!("{} GiB", memory / 1024 / 1024);
    } else if memory >= 1024 {
        // Mebibytes
        return format!("{} MiB", memory / 1024);
    } else {
        // Kibibytes
        return format!("{} KiB", memory);
    }
}

fn print_network_info(net_info_list: Vec<NetworkInterfaceInfo>) {
    for nic in net_info_list {
        colors::print_color(&TerminalColors::BoldPurple, "IP: ");
        colors::println_color(
            &TerminalColors::White,
            format!(
                "{} {}",
                colors::wrap_color(&TerminalColors::White, &nic.address),
                colors::wrap_color(
                    &TerminalColors::BoldGreen,
                    format!("[{}/{}]", nic.name, nic.address_type)
                )
            ),
        );
    }
}

fn print_memory_info(memory_info: MemoryInfo) {
    colors::print_color(&TerminalColors::BoldBlue, "Memory: ");

    let formatted_used_mem = format_memory_amount(memory_info.used_mem);
    let formatted_total_mem = format_memory_amount(memory_info.total_mem);

    colors::println_color(
        &TerminalColors::White,
        format!("{} / {}", formatted_used_mem, formatted_total_mem),
    );

    colors::print_color(&TerminalColors::BoldRed, "Swap: ");

    let formatted_used_swap = format_memory_amount(memory_info.used_swap);
    let formatted_total_swap = format_memory_amount(memory_info.total_swap);

    colors::println_color(
        &TerminalColors::White,
        format!("{} / {}", formatted_used_swap, formatted_total_swap),
    );
}

fn print_gpu_info(gpus_info: &Option<Vec<GPUInfo>>) {
    if let Some(gpus) = &gpus_info {
        for gpu in gpus {
            let gpu_name = &gpu.gpu_name;
            let vendor_color = match gpu.gpu_vendor {
                hardware::GPUVendor::AMD => &TerminalColors::BoldRed,
                hardware::GPUVendor::Intel => &TerminalColors::BoldBlue,
                hardware::GPUVendor::Nvidia => &TerminalColors::BoldGreen,
                _ => &TerminalColors::White,
            };

            let colorized_gpu_name = gpu_name.replace(
                gpu.gpu_vendor.to_str(),
                &colors::wrap_color_with_end(
                    &vendor_color,
                    &gpu.gpu_vendor.to_str(),
                    &TerminalColors::White,
                ),
            );

            colors::print_color(&TerminalColors::BoldPurple, "GPU: ");
            colors::print_color(&TerminalColors::White, colorized_gpu_name);
            colors::print_color(
                &TerminalColors::BoldCyan,
                if gpu.is_discrete {
                    " [Discrete]"
                } else {
                    " [Integrated]"
                },
            );

            if let Some(vram) = &gpu.vram {
                colors::print_color(&TerminalColors::BoldCyan, format!(" [VRAM: {}]", vram));
            }

            println!("");
        }
    }
}

fn print_cpu_info(cpu_info: &CPUInfo) {
    let cpu_name = cpu_info.cpu_name.clone();
    colors::print_color(&TerminalColors::BoldCyan, "CPU: ");

    let vendor_color = match cpu_info.vendor {
        hardware::CPUVendor::AMD => &TerminalColors::BoldRed,
        hardware::CPUVendor::Intel => &TerminalColors::BoldBlue,
        hardware::CPUVendor::Nvidia => &TerminalColors::BoldGreen,
        _ => &TerminalColors::White,
    };

    let colorized_cpu_name = cpu_name.replace(
        &cpu_info.vendor.to_str(),
        &colors::wrap_color_with_end(
            &vendor_color,
            cpu_info.vendor.to_str(),
            &TerminalColors::White,
        ),
    );

    colors::print_color(&TerminalColors::White, colorized_cpu_name);

    if cpu_info.is_smt_enabled {
        colors::print_color(&TerminalColors::BoldGreen, " [+SMT]");
    } else {
        colors::print_color(&TerminalColors::BoldRed, " [-SMT]");
    }

    if cpu_info.is_turbo_boost_enabled {
        colors::print_color(&TerminalColors::BoldGreen, " [+Turbo]");
    } else {
        colors::print_color(&TerminalColors::BoldRed, " [-Turbo]");
    }

    println!("");
}

fn print_npu_info(npu_info: &NPUInfo) {
    let npu_name = npu_info.npu_name.clone();
    colors::print_color(&TerminalColors::BoldGreen, "NPU: ");

    colors::print_color(&TerminalColors::White, npu_name);

    println!("");
}

fn print_package_info(
    wrapped_pkg_info: &Result<pkg::SystemPackageStats, String>,
    wrapped_flatpak_info: &Result<usize, &str>,
    wrapped_snap_info: &Result<usize, &str>,
) {
    if let Ok(pkg_info) = wrapped_pkg_info {
        let flatpak_pkg_count = match wrapped_flatpak_info {
            Ok(flatpak_pkg_count) => flatpak_pkg_count,
            Err(_) => &0,
        };

        let snap_pkg_count = match wrapped_snap_info {
            Ok(snap_pkg_count) => snap_pkg_count,
            Err(_) => &0,
        };

        colors::print_color(&TerminalColors::BoldBlue, "Packages: ");
        colors::print_color(&TerminalColors::BoldPurple, pkg_info.pkg_mgr_name.clone());
        colors::print_color(&TerminalColors::White, format!(" ({})", pkg_info.pkg_count));

        if flatpak_pkg_count > &0 {
            print!(", ");
            colors::print_color(&TerminalColors::BoldCyan, "flatpak");

            colors::print_color(&TerminalColors::White, format!(" ({})", flatpak_pkg_count));
        }

        if snap_pkg_count > &0 {
            print!(", ");
            colors::print_color(&TerminalColors::BoldCyan, "snap");

            colors::print_color(&TerminalColors::White, format!(" ({})", snap_pkg_count));
        }

        println!("");
    }
}

fn make_separator(length: usize) -> String {
    let mut str: String = String::with_capacity(length);
    for i in 0..length {
        str.insert(i, '-');
    }
    return str;
}

fn print_header() {
    let user = std::env!("USER");
    let header = format!("{}@{}", user, read_host_name());
    let header_sep = make_separator(header.len());

    colors::println_color(&TerminalColors::BoldBlue, header);
    colors::println_color(&TerminalColors::White, header_sep);
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
                    .expect("Can't read os name.")
                    .trim()
                    .trim_start_matches("\"")
                    .trim_end_matches("\"")
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

fn read_uptime() -> String {
    let uptime = std::fs::read_to_string("/proc/uptime");
    if let Ok(contents) = uptime {
        match contents.split(' ').collect::<Vec<_>>().as_array() {
            Some([uptime, _]) => {
                let total_seconds = uptime.parse::<f64>().expect("N/A");

                let mut result = String::new();

                let days = f64::round(total_seconds / 86400.0);
                let days_rem = total_seconds % 86400.0;
                let hours = f64::round(days_rem / 3600.0);
                let hours_rem = days_rem % 3600.0;
                let minutes = f64::round(hours_rem / 60.0);
                // let minutes_rem = hours_rem % 60.0;
                // let seconds = f64::round(minutes_rem / 60.0);

                // format!("{} days, {} hours, {} minutes", days, hours, minutes)

                if days > 0.0 {
                    result.push_str(format!("{} days, ", days).as_str());
                }

                if hours > 0.0 {
                    result.push_str(format!("{} hours, ", hours).as_str());
                }

                result.push_str(format!("{} minutes", minutes).as_str());

                result
            }
            None => String::from("N/A"),
        }
    } else {
        String::from("N/A")
    }
}

fn detect_terminal() -> String {
    let env_vars = std::env::vars();

    let mut terminal = String::from("Unknown");
    for (k, v) in env_vars {
        if k == "KITTY_PID" {
            terminal = "kitty".to_owned();
            break;
        } else if k == "KONSOLE_VERSION" {
            terminal = "konsole".to_owned();
            break;
        } else if k == "ALACRITTY_WINDOW_ID" {
            terminal = "alacritty".to_owned();
            break;
        } else if k == "TERM_PROGRAM" {
            let t = v.to_owned();
            terminal = t;
            break;
        }
    }

    terminal.to_string()
}
