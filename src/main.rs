mod colors;
mod graphics_env;
mod shell;

use colors::TerminalColors;
use shell::get_shell;
fn main() {
    colors::print_color(TerminalColors::Blue, "Hostname: ");
    colors::println_color(TerminalColors::Black, read_host_name());

    colors::print_color(TerminalColors::Green, "OS Name: ");
    colors::println_color(TerminalColors::Black, read_os_name());

    colors::print_color(TerminalColors::Red, "Kernel: ");
    colors::println_color(TerminalColors::Black, read_kernel());

    colors::print_color(TerminalColors::Yellow, "Uptime: ");
    colors::println_color(TerminalColors::Black, read_uptime());

    colors::print_color(TerminalColors::LightBlue, "Shell: ");
    colors::println_color(TerminalColors::Black, get_shell());

    let desktop_env = graphics_env::get_desktop_env();

    colors::print_color(TerminalColors::LightPurple, "Session: ");
    colors::println_color(TerminalColors::Black, desktop_env.session_type.as_str());

    colors::print_color(TerminalColors::LightGreen, "DE: ");
    colors::println_color(
        TerminalColors::Black,
        format!(
            "{} ({})",
            desktop_env.desktop_env, desktop_env.desktop_env_version
        ),
    );
    colors::print_color(TerminalColors::Green, "WM: ");
    colors::println_color(
        TerminalColors::Black,
        format!(
            "{} ({})",
            desktop_env.window_manager, desktop_env.window_manager_version
        ),
    );
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
