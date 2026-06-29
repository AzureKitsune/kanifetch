pub enum DesktopSessionType {
    X11,
    Wayland,
    Unknown,
}

impl DesktopSessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DesktopSessionType::Wayland => "wayland",
            DesktopSessionType::X11 => "x11",
            DesktopSessionType::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> DesktopSessionType {
        match value.to_lowercase().trim() {
            "x11" => DesktopSessionType::X11,
            "wayland" => DesktopSessionType::Wayland,
            _ => DesktopSessionType::Unknown,
        }
    }
}

pub struct DesktopEnv {
    pub desktop_env: String,
    pub desktop_env_version: String,
    pub window_manager: String,
    pub window_manager_version: String,
    pub session_type: DesktopSessionType,
}

pub fn get_desktop_env() -> DesktopEnv {
    let session_type: String =
        std::env::var("XDG_SESSION_TYPE").expect("Unable to get graphical session type.");

    let current_session: String =
        std::env::var("XDG_CURRENT_DESKTOP").expect("Unable to get desktop type.");

    let formatted_current_session = match current_session.to_lowercase().as_str() {
        "kde" => "KDE Plasma".to_string(),
        "ubuntu:gnome" => "GNOME".to_string(),
        _ => current_session.to_string(),
    };

    let mut desktop_env_version: String = String::new();

    let (window_manager, window_manager_ver) = match formatted_current_session.as_str() {
        "KDE Plasma" => {
            let kwin_ver = String::from_utf8(
                std::process::Command::new("kwin")
                    .arg("--version")
                    .output()
                    .expect("Unable to get kwin version.")
                    .stdout,
            )
            .expect("Unable to parse kwin version output.")
            .trim()
            .split(' ')
            .nth(1)
            .expect("Unable to get DE version.")
            .to_string();

            desktop_env_version = kwin_ver.to_string();
            ("kwin", kwin_ver)
        }
        "GNOME" => {
            // untested
            let mutter_ver = String::from_utf8(
                std::process::Command::new("gnome-shell")
                    .arg("--version")
                    .output()
                    .expect("Unable to get gnome-shell (mutter) version.")
                    .stdout,
            )
            .expect("Unable to parse gnome-shell (mutter) version output.")
            .trim()
            .to_string();

            desktop_env_version = mutter_ver.to_string();
            ("mutter", mutter_ver)
        }
        _ => ("Unknown", String::from("Unknown")),
    };

    DesktopEnv {
        desktop_env: formatted_current_session,
        desktop_env_version: desktop_env_version,
        window_manager: window_manager.to_string(),
        window_manager_version: window_manager_ver,
        session_type: DesktopSessionType::from_str(session_type.as_str()),
    }
}
