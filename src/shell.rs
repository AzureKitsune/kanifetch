use std::env::args;

fn get_shell_version(shell: &str) -> String {
    String::from_utf8(
        std::process::Command::new(shell)
            .arg("--version")
            .output()
            .expect("Unknown")
            .stdout,
    )
    .expect("Unable to read output")
}

pub fn get_shell() -> String {
    let shell = std::env!("SHELL"); // TODO this is for the login shell. if a new shell is spawned and then this gets executed, it will report the wrong shell.

    match shell {
        "/usr/bin/zsh" | "/bin/zsh" => get_shell_version("zsh").trim().to_string(),
        "/usr/bin/bash" | "/bin/bash" => {
            let output = get_shell_version("bash");
            let output_lines: Vec<_> = output.split_terminator("\n").collect();
            let first_line = output_lines.first().expect("Unknown");
            let first_line_clauses: Vec<&str> = first_line.split_terminator(",").collect();
            format!(
                "bash {}",
                first_line_clauses
                    .iter()
                    .nth(1)
                    .expect("Unknown")
                    .to_string()
                    .trim()
            )
        }
        "/usr/bin/fish" | "/bin/fish" => {
            let output = get_shell_version("fish");
            let clauses: Vec<&str> = output.split_terminator(",").collect();
            format!(
                "fish {}",
                clauses.iter().nth(1).expect("Unknown").to_string().trim()
            )
        }

        _ => shell.to_string(),
    }
}
