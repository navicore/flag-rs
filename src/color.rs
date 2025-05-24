//! Simple ANSI color support with zero dependencies
//!
//! This module provides basic color support for terminal output without
//! requiring any external dependencies. It automatically detects whether
//! colors should be used based on terminal capabilities and environment.
//!
//! # Features
//!
//! - Automatic TTY detection
//! - Respects `NO_COLOR` environment variable
//! - Zero dependencies - uses only Rust standard library
//! - Common ANSI colors and styles
//!
//! # Examples
//!
//! ```
//! use flag::color;
//!
//! // Using convenience functions
//! println!("{}", color::red("Error: Something went wrong"));
//! println!("{}", color::green("Success!"));
//! println!("{}", color::bold("Important message"));
//!
//! // Using Style directly for more control
//! use flag::color::Style;
//! println!("{}", Style::YELLOW.paint("Warning: Check this out"));
//! ```

use std::env;
use std::io::{self, IsTerminal};

/// ANSI style configuration
///
/// Represents a text style with ANSI escape codes for coloring terminal output.
pub struct Style {
    prefix: &'static str,
    suffix: &'static str,
}

impl Style {
    /// ANSI reset code to clear all styles
    pub const RESET: &'static str = "\x1b[0m";

    /// Red color style
    pub const RED: Self = Self {
        prefix: "\x1b[31m",
        suffix: Self::RESET,
    };

    /// Green color style
    pub const GREEN: Self = Self {
        prefix: "\x1b[32m",
        suffix: Self::RESET,
    };

    /// Yellow color style
    pub const YELLOW: Self = Self {
        prefix: "\x1b[33m",
        suffix: Self::RESET,
    };

    /// Blue color style
    pub const BLUE: Self = Self {
        prefix: "\x1b[34m",
        suffix: Self::RESET,
    };

    /// Magenta color style
    pub const MAGENTA: Self = Self {
        prefix: "\x1b[35m",
        suffix: Self::RESET,
    };

    /// Cyan color style
    pub const CYAN: Self = Self {
        prefix: "\x1b[36m",
        suffix: Self::RESET,
    };

    /// Bold text style
    pub const BOLD: Self = Self {
        prefix: "\x1b[1m",
        suffix: Self::RESET,
    };

    /// Dim/faint text style
    pub const DIM: Self = Self {
        prefix: "\x1b[2m",
        suffix: Self::RESET,
    };

    /// Applies this style to the given text
    ///
    /// The style is only applied if colors are enabled (stdout is a TTY
    /// and `NO_COLOR` is not set).
    ///
    /// # Arguments
    ///
    /// * `text` - The text to style
    ///
    /// # Returns
    ///
    /// The styled text if colors are enabled, otherwise the original text
    #[must_use]
    pub fn paint(&self, text: &str) -> String {
        if should_colorize() {
            format!("{}{}{}", self.prefix, text, self.suffix)
        } else {
            text.to_string()
        }
    }
}

/// Determines whether output should be colorized
///
/// Returns `true` if:
/// - stdout is a terminal (TTY)
/// - `NO_COLOR` environment variable is not set
///
/// This follows the `NO_COLOR` standard: <https://no-color.org/>
#[must_use]
pub fn should_colorize() -> bool {
    // Respect `NO_COLOR` environment variable
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if stdout is a terminal
    io::stdout().is_terminal()
}

/// Colors text red (typically for errors)
#[must_use]
pub fn red(text: &str) -> String {
    Style::RED.paint(text)
}

/// Colors text green (typically for success)
#[must_use]
pub fn green(text: &str) -> String {
    Style::GREEN.paint(text)
}

/// Colors text yellow (typically for warnings)
#[must_use]
pub fn yellow(text: &str) -> String {
    Style::YELLOW.paint(text)
}

/// Colors text blue (typically for information)
#[must_use]
pub fn blue(text: &str) -> String {
    Style::BLUE.paint(text)
}

/// Colors text cyan (typically for highlights)
#[must_use]
pub fn cyan(text: &str) -> String {
    Style::CYAN.paint(text)
}

/// Makes text bold (typically for emphasis)
#[must_use]
pub fn bold(text: &str) -> String {
    Style::BOLD.paint(text)
}

/// Makes text dim (typically for less important information)
#[must_use]
pub fn dim(text: &str) -> String {
    Style::DIM.paint(text)
}
