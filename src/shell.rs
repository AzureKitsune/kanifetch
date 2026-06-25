fn get_shell_version(shell: &str) -> String {
    String::from_utf8(
        std::process::Command::new(shell)
            .arg("--version")
            .output()
            .expect("Unknown")
            .stdout,
    )
    .expect("Unable to read shell version output.")
}

// pub fn get_login_shell() -> String {
//     std::env!("SHELL").to_string()
// }

pub fn get_shell() -> String {
    let ppid = std::os::unix::process::parent_id();
    let ppid_path =
        std::fs::read_link(format!("/proc/{ppid}/exe")).expect("Unable to determine shell.");

    let shell = ppid_path.to_str().expect("Unable to determine shell.");

    match shell.trim() {
        "/usr/bin/zsh" | "/bin/zsh" => {
            let output = get_shell_version("zsh");
            let trimmed_output = output.trim();
            let output_parts: Vec<_> = trimmed_output.split("(").collect();
            let name_and_version = output_parts.get(0).expect("Unable to get zsh version.");

            return name_and_version.trim().to_string();
        }
        "/usr/bin/bash" | "/bin/bash" => {
            let output = get_shell_version("bash");
            let output_lines: Vec<_> = output.split_terminator("\n").collect();
            let first_line = output_lines.first().expect("Unknown");
            let first_line_clauses: Vec<&str> = first_line.split_terminator(",").collect();

            return format!(
                "bash {}",
                first_line_clauses
                    .iter()
                    .nth(1)
                    .expect("Unknown")
                    .to_string()
                    .trim()
            );
        }
        "/usr/bin/fish" | "/bin/fish" => {
            let output = get_shell_version("fish");
            let clauses: Vec<&str> = output.split_terminator(",").collect();

            return format!(
                "fish {}",
                clauses.iter().nth(1).expect("Unknown").to_string().trim()
            );
        }

        _ => shell.trim().to_string(),
    }
}
