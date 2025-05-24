/// Simple ANSI color support with zero dependencies
///
/// This module provides basic color support for terminal output.
/// It checks if stdout is a TTY and respects NO_COLOR environment variable.
use std::env;
use std::io::{self, IsTerminal};

pub struct Style {
    prefix: &'static str,
    suffix: &'static str,
}

impl Style {
    pub const RESET: &'static str = "\x1b[0m";

    pub const RED: Style = Style {
        prefix: "\x1b[31m",
        suffix: Self::RESET,
    };
    pub const GREEN: Style = Style {
        prefix: "\x1b[32m",
        suffix: Self::RESET,
    };
    pub const YELLOW: Style = Style {
        prefix: "\x1b[33m",
        suffix: Self::RESET,
    };
    pub const BLUE: Style = Style {
        prefix: "\x1b[34m",
        suffix: Self::RESET,
    };
    pub const MAGENTA: Style = Style {
        prefix: "\x1b[35m",
        suffix: Self::RESET,
    };
    pub const CYAN: Style = Style {
        prefix: "\x1b[36m",
        suffix: Self::RESET,
    };
    pub const BOLD: Style = Style {
        prefix: "\x1b[1m",
        suffix: Self::RESET,
    };
    pub const DIM: Style = Style {
        prefix: "\x1b[2m",
        suffix: Self::RESET,
    };

    pub fn paint(&self, text: &str) -> String {
        if should_colorize() {
            format!("{}{}{}", self.prefix, text, self.suffix)
        } else {
            text.to_string()
        }
    }
}

/// Check if we should colorize output
pub fn should_colorize() -> bool {
    // Respect NO_COLOR environment variable
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if stdout is a terminal
    io::stdout().is_terminal()
}

/// Convenience functions for common colors
pub fn red(text: &str) -> String {
    Style::RED.paint(text)
}

pub fn green(text: &str) -> String {
    Style::GREEN.paint(text)
}

pub fn yellow(text: &str) -> String {
    Style::YELLOW.paint(text)
}

pub fn blue(text: &str) -> String {
    Style::BLUE.paint(text)
}

pub fn cyan(text: &str) -> String {
    Style::CYAN.paint(text)
}

pub fn bold(text: &str) -> String {
    Style::BOLD.paint(text)
}

pub fn dim(text: &str) -> String {
    Style::DIM.paint(text)
}
