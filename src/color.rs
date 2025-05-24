/// Simple ANSI color support with zero dependencies
///
/// This module provides basic color support for terminal output.
/// It checks if stdout is a TTY and respects `NO_COLOR` environment variable.
use std::env;
use std::io::{self, IsTerminal};

pub struct Style {
    prefix: &'static str,
    suffix: &'static str,
}

impl Style {
    pub const RESET: &'static str = "\x1b[0m";

    pub const RED: Self = Self {
        prefix: "\x1b[31m",
        suffix: Self::RESET,
    };
    pub const GREEN: Self = Self {
        prefix: "\x1b[32m",
        suffix: Self::RESET,
    };
    pub const YELLOW: Self = Self {
        prefix: "\x1b[33m",
        suffix: Self::RESET,
    };
    pub const BLUE: Self = Self {
        prefix: "\x1b[34m",
        suffix: Self::RESET,
    };
    pub const MAGENTA: Self = Self {
        prefix: "\x1b[35m",
        suffix: Self::RESET,
    };
    pub const CYAN: Self = Self {
        prefix: "\x1b[36m",
        suffix: Self::RESET,
    };
    pub const BOLD: Self = Self {
        prefix: "\x1b[1m",
        suffix: Self::RESET,
    };
    pub const DIM: Self = Self {
        prefix: "\x1b[2m",
        suffix: Self::RESET,
    };

    #[must_use]
    pub fn paint(&self, text: &str) -> String {
        if should_colorize() {
            format!("{}{}{}", self.prefix, text, self.suffix)
        } else {
            text.to_string()
        }
    }
}

/// Check if we should colorize output
#[must_use]
pub fn should_colorize() -> bool {
    // Respect `NO_COLOR` environment variable
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if stdout is a terminal
    io::stdout().is_terminal()
}

/// Convenience functions for common colors
#[must_use]
pub fn red(text: &str) -> String {
    Style::RED.paint(text)
}

#[must_use]
pub fn green(text: &str) -> String {
    Style::GREEN.paint(text)
}

#[must_use]
pub fn yellow(text: &str) -> String {
    Style::YELLOW.paint(text)
}

#[must_use]
pub fn blue(text: &str) -> String {
    Style::BLUE.paint(text)
}

#[must_use]
pub fn cyan(text: &str) -> String {
    Style::CYAN.paint(text)
}

#[must_use]
pub fn bold(text: &str) -> String {
    Style::BOLD.paint(text)
}

#[must_use]
pub fn dim(text: &str) -> String {
    Style::DIM.paint(text)
}
