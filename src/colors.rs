// COLORS - https://stackoverflow.com/a/5947802

use core::fmt;

pub enum TerminalColors {
    Black,
    Red,
    Green,
    BrownOrange,
    Blue,
    Purple,
    Cyan,
    LightGray,
    DarkGray,
    LightRed,
    LightGreen,
    Yellow,
    LightBlue,
    LightPurple,
    LightCyan,
    White,
    NoColor, // No Color
}

impl TerminalColors {
    pub fn as_str(&self) -> &'static str {
        match self {
            TerminalColors::Black => "\x1b[0;30m",
            TerminalColors::Red => "\x1b[0;31m",
            TerminalColors::Green => "\x1b[0;32m",
            TerminalColors::BrownOrange => "\x1b[0;33m",
            TerminalColors::Blue => "\x1b[0;34m",
            TerminalColors::Purple => "\x1b[0;35m",
            TerminalColors::Cyan => "\x1b[0;36m",
            TerminalColors::LightGray => "\x1b[0;37m",
            TerminalColors::DarkGray => "\x1b[1;30m",
            TerminalColors::LightRed => "\x1b[1;31m",
            TerminalColors::LightGreen => "\x1b[1;32m",
            TerminalColors::Yellow => "\x1b[1;33m",
            TerminalColors::LightBlue => "\x1b[1;34m",
            TerminalColors::LightPurple => "\x1b[1;35m",
            TerminalColors::LightCyan => "\x1b[1;36m",
            TerminalColors::White => "\x1b[1;37m",
            TerminalColors::NoColor => "\x1b[0m", // No Color
        }
    }
}

impl fmt::Display for TerminalColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
