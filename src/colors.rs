// COLORS - https://stackoverflow.com/a/5947802

use core::fmt;

#[allow(unused)]
pub enum TerminalColors {
    Black,
    Red,
    Green,
    BrownOrange,
    Blue,
    Purple,
    Cyan,
    BoldGray,
    DarkGray,
    BoldRed,
    BoldGreen,
    Yellow,
    BoldBlue,
    BoldPurple,
    BoldCyan,
    White,
    NoColor, // No Color
}

impl TerminalColors {
    // Enum -> str
    pub fn as_str(&self) -> &'static str {
        match self {
            TerminalColors::Black => "\x1b[0;30m",
            TerminalColors::Red => "\x1b[0;31m",
            TerminalColors::Green => "\x1b[0;32m",
            TerminalColors::BrownOrange => "\x1b[0;33m",
            TerminalColors::Blue => "\x1b[0;34m",
            TerminalColors::Purple => "\x1b[0;35m",
            TerminalColors::Cyan => "\x1b[0;36m",
            TerminalColors::BoldGray => "\x1b[0;37m",
            TerminalColors::DarkGray => "\x1b[1;30m",
            TerminalColors::BoldRed => "\x1b[1;31m",
            TerminalColors::BoldGreen => "\x1b[1;32m",
            TerminalColors::Yellow => "\x1b[1;33m",
            TerminalColors::BoldBlue => "\x1b[1;34m",
            TerminalColors::BoldPurple => "\x1b[1;35m",
            TerminalColors::BoldCyan => "\x1b[1;36m",
            TerminalColors::White => "\x1b[1;37m",
            TerminalColors::NoColor => "\x1b[0m", // No Color
        }
    }
}

impl fmt::Display for TerminalColors {
    // Needed to allow usage in format strings?
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// std::fmt::Display means its convertable to a string?
pub fn print_color<T: std::fmt::Display>(color: &TerminalColors, msg: T) {
    print!("{}{}", color.as_str(), msg);
}

pub fn println_color<T: std::fmt::Display>(color: &TerminalColors, msg: T) {
    print_color(color, format!("{}\n", msg));
}
