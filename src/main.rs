mod colors;
mod graphics_env;
mod hardware;
mod network;
mod shell;

use colors::TerminalColors;
use hardware::get_hardware_info;
use shell::get_shell;
fn main() {
    print_header();

    colors::print_color(TerminalColors::BoldGreen, "OS Name: ");
    colors::println_color(TerminalColors::White, read_os_name());

    colors::print_color(TerminalColors::BoldRed, "Kernel: ");
    colors::println_color(TerminalColors::White, read_kernel());

    colors::print_color(TerminalColors::BoldCyan, "Uptime: ");
    colors::println_color(TerminalColors::White, read_uptime());

    colors::print_color(TerminalColors::BoldBlue, "Shell: ");
    colors::println_color(TerminalColors::White, get_shell());

    colors::print_color(TerminalColors::BoldRed, "Terminal: ");
    colors::println_color(TerminalColors::White, detect_terminal());

    let desktop_env = graphics_env::get_desktop_env();

    colors::print_color(TerminalColors::BoldPurple, "Session: ");
    colors::println_color(TerminalColors::White, desktop_env.session_type.as_str());

    colors::print_color(TerminalColors::BoldGreen, "DE: ");
    colors::println_color(
        TerminalColors::White,
        format!(
            "{} ({})",
            desktop_env.desktop_env, desktop_env.desktop_env_version
        ),
    );
    colors::print_color(TerminalColors::BoldPurple, "WM: ");
    colors::println_color(
        TerminalColors::White,
        format!(
            "{} ({})",
            desktop_env.window_manager, desktop_env.window_manager_version
        ),
    );

    let net_info = network::get_ipv4_info();
    match net_info {
        Ok(item) => {
            colors::print_color(
                TerminalColors::BoldPurple,
                format!("NET ({}/{}): ", item.name, item.address_type),
            );
            colors::println_color(TerminalColors::White, item.address);
        }
        Err(_) => {}
    }

    let hardware_info = get_hardware_info();
    colors::print_color(TerminalColors::BoldRed, "CPU: ");
    colors::println_color(TerminalColors::White, hardware_info.cpu_name);

    if let Some(igpu_name) = hardware_info.integrated_gpu_name {
        colors::print_color(TerminalColors::BoldBlue, "GPU: ");
        colors::print_color(TerminalColors::White, igpu_name);
        colors::println_color(TerminalColors::BoldCyan, " [Integrated]");
    }
}

fn make_separator(length: usize) -> String {
    //TODO figure out a better way to do this
    let mut header_sep: Vec<&str> = Vec::new();
    for _ in 0..length {
        header_sep.push("-");
    }
    header_sep.join("")
}

fn print_header() {
    let user = std::env!("USER");
    let header = format!("{}@{}", user, read_host_name());
    let header_sep = make_separator(header.len());

    colors::println_color(TerminalColors::BoldBlue, header);
    colors::println_color(TerminalColors::White, header_sep);
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
