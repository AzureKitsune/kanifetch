use which::which;

pub struct SystemPackageStats {
    pub pkg_mgr_name: String,
    pub pkg_count: usize,
}

pub fn get_system_pkg_mgr() -> Result<String, &'static str> {
    if which("apt").is_ok() {
        return Ok(String::from("apt"));
    }

    if which("dnf").is_ok() {
        return Ok(String::from("dnf"));
    }

    if which("pacman").is_ok() {
        return Ok(String::from("pacman"));
    }

    Err("Unsupported package manager.")
}

pub fn get_installed_system_pkg() -> Result<SystemPackageStats, String> {
    let sys_pkg_mgr = get_system_pkg_mgr();

    if sys_pkg_mgr.is_err() {
        return Err::<SystemPackageStats, String>("Unsupported package manager".to_string());
    }

    let unwrapped_pkg_mgr = sys_pkg_mgr.unwrap();
    match unwrapped_pkg_mgr.as_str() {
        "dnf" => {
            let proc = std::process::Command::new("dnf")
                .arg("--cacheonly")
                .arg("list")
                .arg("--installed")
                .output();

            let pkg_line_count = proc
                .and_then(|output| {
                    let stdout = String::from_utf8(output.stdout).unwrap();
                    let lines: Vec<_> = stdout.split("\n").collect();
                    Ok(lines.len())
                })
                .or_else(|_| Err("Unable to get package count from dnf."));

            return Ok(SystemPackageStats {
                pkg_mgr_name: String::from("dnf"),
                pkg_count: pkg_line_count.unwrap_or_else(|_| 0 as usize),
            });
        }
        "apt" => {
            let proc = std::process::Command::new("dpkg-query")
                .arg("-W")
                .arg("-f='${Package}\n'")
                .output();

            let pkg_line_count = proc
                .and_then(|output| {
                    let stdout = String::from_utf8(output.stdout).unwrap();
                    let lines: Vec<_> = stdout.split("\n").collect();
                    Ok(lines.len())
                })
                .or_else(|_| Err("Unable to get package count from apt."));

            return Ok(SystemPackageStats {
                pkg_mgr_name: String::from("apt"),
                pkg_count: pkg_line_count.unwrap_or_else(|_| 0 as usize),
            });
        }
        _ => Err(format!(
            "Unsupported package manager: {}",
            unwrapped_pkg_mgr
        )),
    }
}

pub fn get_installed_flatpaks() -> Result<usize, &'static str> {
    if which("flatpak").is_ok() {
        let proc = std::process::Command::new("flatpak").arg("list").output();

        let flatpak_list_count = proc
            .and_then(|output| {
                let stdout = String::from_utf8(output.stdout).unwrap();
                let lines: Vec<_> = stdout.split("\n").collect();
                Ok(lines.len() - 1) // Subtract one for the header line
            })
            .or_else(|_| Err("Unable to get package count from flatpak."));
        return flatpak_list_count;
    }

    Err("Flatpak isn't installed.")
}

pub fn get_installed_snaps() -> Result<usize, &'static str> {
    if which("snap").is_ok() {
        let proc = std::process::Command::new("snap").arg("list").output();

        let snap_list_count = proc
            .and_then(|output| {
                let stdout = String::from_utf8(output.stdout).unwrap();
                let lines: Vec<_> = stdout.split("\n").collect();
                Ok(lines.len() - 1) // Subtract one for the header line
            })
            .or_else(|_| Err("Unable to get package count from flatpak."));
        return snap_list_count;
    }

    Err("Snap isn't installed.")
}
