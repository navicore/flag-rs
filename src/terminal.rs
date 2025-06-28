//! Terminal utilities for enhanced CLI output
//!
//! This module provides utilities for working with terminal capabilities,
//! including width detection, text wrapping, and improved formatting.

use std::env;

/// Default terminal width to use when detection fails
const DEFAULT_TERMINAL_WIDTH: usize = 80;

/// Minimum terminal width to prevent text from being too cramped
const MINIMUM_TERMINAL_WIDTH: usize = 40;

/// Maximum terminal width to prevent lines from being too long
const MAXIMUM_TERMINAL_WIDTH: usize = 120;

/// Detects the current terminal width
///
/// This function attempts to determine the terminal width using multiple methods:
/// 1. `COLUMNS` environment variable (set by most shells)
/// 2. Platform-specific terminal size detection
/// 3. Falls back to a sensible default
///
/// The returned width is clamped between reasonable minimum and maximum values
/// to ensure readable output across different terminal sizes.
///
/// # Examples
///
/// ```rust
/// use flag_rs::terminal::get_terminal_width;
///
/// let width = get_terminal_width();
/// println!("Terminal width: {}", width);
/// ```
#[must_use]
pub fn get_terminal_width() -> usize {
    // First, try the COLUMNS environment variable
    if let Ok(columns_str) = env::var("COLUMNS") {
        if let Ok(columns) = columns_str.parse::<usize>() {
            return clamp_width(columns);
        }
    }

    // Try platform-specific detection
    if let Some(width) = detect_terminal_width_platform() {
        return clamp_width(width);
    }

    // Fall back to default
    DEFAULT_TERMINAL_WIDTH
}

/// Platform-specific terminal width detection
fn detect_terminal_width_platform() -> Option<usize> {
    #[cfg(unix)]
    {
        detect_terminal_width_unix()
    }
    
    #[cfg(windows)]
    {
        detect_terminal_width_windows()
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(unix)]
fn detect_terminal_width_unix() -> Option<usize> {
    use std::io::IsTerminal;
    
    // Only try to detect width if we're actually connected to a terminal
    if !std::io::stdout().is_terminal() {
        return None;
    }
    
    // Try to get terminal size using TIOCGWINSZ ioctl
    // This is a simplified implementation - a full implementation would use libc
    // For now, we'll rely on COLUMNS env var and fall back to default
    None
}

#[cfg(windows)]
fn detect_terminal_width_windows() -> Option<usize> {
    // On Windows, we could use GetConsoleScreenBufferInfo
    // For now, we'll rely on COLUMNS env var and fall back to default
    None
}

/// Clamps the terminal width to reasonable bounds
fn clamp_width(width: usize) -> usize {
    width.clamp(MINIMUM_TERMINAL_WIDTH, MAXIMUM_TERMINAL_WIDTH)
}

/// Wraps text to fit within the specified width
///
/// This function intelligently wraps text by:
/// - Breaking at word boundaries when possible
/// - Preserving existing line breaks
/// - Handling indentation for subsequent lines
///
/// # Arguments
///
/// * `text` - The text to wrap
/// * `width` - The maximum line width
/// * `indent` - Optional indentation for continuation lines
///
/// # Examples
///
/// ```rust
/// use flag_rs::terminal::wrap_text;
///
/// let text = "This is a very long line that needs to be wrapped to fit within the terminal width.";
/// let wrapped = wrap_text(text, 40, Some(4));
/// println!("{}", wrapped);
/// ```
pub fn wrap_text(text: &str, width: usize, indent: Option<usize>) -> String {
    if text.is_empty() || width == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    let indent_str = " ".repeat(indent.unwrap_or(0));
    let mut first_line = true;

    for paragraph in text.split('\n') {
        if !first_line {
            result.push('\n');
        }
        first_line = false;

        if paragraph.trim().is_empty() {
            continue;
        }

        let mut current_line = String::new();
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        
        for word in &words {
            let space_needed = if current_line.is_empty() { 0 } else { 1 };
            let line_with_word_len = current_line.len() + space_needed + word.len();
            
            if line_with_word_len <= width || current_line.is_empty() {
                // Word fits on current line
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                // Word doesn't fit, start new line
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&current_line);
                
                current_line = format!("{}{}", indent_str, word);
            }
        }
        
        // Add the last line
        if !current_line.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
        }
    }

    result
}

/// Wraps text to fit the current terminal width
///
/// This is a convenience function that automatically detects the terminal width
/// and wraps the text accordingly.
///
/// # Arguments
///
/// * `text` - The text to wrap
/// * `indent` - Optional indentation for continuation lines
///
/// # Examples
///
/// ```rust
/// use flag_rs::terminal::wrap_text_to_terminal;
///
/// let text = "This is a very long line that will be wrapped to fit the current terminal.";
/// let wrapped = wrap_text_to_terminal(text, Some(2));
/// println!("{}", wrapped);
/// ```
pub fn wrap_text_to_terminal(text: &str, indent: Option<usize>) -> String {
    let width = get_terminal_width();
    wrap_text(text, width, indent)
}

/// Formats help text with proper alignment and wrapping
///
/// This function formats help text entries (like flag descriptions) with
/// consistent alignment and automatic text wrapping.
///
/// # Arguments
///
/// * `left_column` - The left column content (e.g., flag name)
/// * `right_column` - The right column content (e.g., description)
/// * `left_width` - Width of the left column
/// * `total_width` - Total width available
///
/// # Examples
///
/// ```rust
/// use flag_rs::terminal::format_help_entry;
///
/// let formatted = format_help_entry(
///     "  -v, --verbose", 
///     "Enable verbose output with detailed logging information",
///     20,
///     80
/// );
/// println!("{}", formatted);
/// ```
pub fn format_help_entry(
    left_column: &str,
    right_column: &str,
    left_width: usize,
    total_width: usize,
) -> String {
    if right_column.is_empty() {
        return left_column.to_string();
    }

    let right_width = total_width.saturating_sub(left_width + 2); // 2 for spacing
    
    if left_column.len() <= left_width {
        // Left column fits, format normally
        let padding = " ".repeat(left_width - left_column.len());
        let wrapped_right = wrap_text(right_column, right_width, Some(left_width + 2));
        
        format!("{}{}  {}", left_column, padding, wrapped_right)
    } else {
        // Left column is too long, put description on next line
        let indent = " ".repeat(left_width + 2);
        let wrapped_right = wrap_text(right_column, right_width, Some(left_width + 2));
        
        format!("{}\n{}{}", left_column, indent, wrapped_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_terminal_width_default() {
        // Should return a reasonable default
        let width = get_terminal_width();
        assert!(width >= MINIMUM_TERMINAL_WIDTH);
        assert!(width <= MAXIMUM_TERMINAL_WIDTH);
    }

    #[test]
    fn test_clamp_width() {
        assert_eq!(clamp_width(10), MINIMUM_TERMINAL_WIDTH);
        assert_eq!(clamp_width(80), 80);
        assert_eq!(clamp_width(200), MAXIMUM_TERMINAL_WIDTH);
    }

    #[test]
    fn test_wrap_text_simple() {
        let text = "This is a test";
        let wrapped = wrap_text(text, 20, None);
        assert_eq!(wrapped, "This is a test");
    }

    #[test]
    fn test_wrap_text_long_line() {
        let text = "This is a very long line that needs to be wrapped";
        let wrapped = wrap_text(text, 20, None);
        
        // Should wrap at word boundaries
        assert!(wrapped.contains('\n'));
        for line in wrapped.lines() {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_wrap_text_with_indent() {
        let text = "This is a very long line that needs to be wrapped with indentation";
        let wrapped = wrap_text(text, 20, Some(4));
        
        let lines: Vec<&str> = wrapped.lines().collect();
        assert!(lines.len() > 1);
        
        // First line should not be indented
        assert!(!lines[0].starts_with("    "));
        
        // Subsequent lines should be indented
        for line in &lines[1..] {
            assert!(line.starts_with("    "));
        }
    }

    #[test]
    fn test_format_help_entry_normal() {
        let result = format_help_entry("  -v, --verbose", "Enable verbose output", 20, 60);
        assert!(result.contains("  -v, --verbose"));
        assert!(result.contains("Enable verbose output"));
    }

    #[test]
    fn test_format_help_entry_long_left() {
        let result = format_help_entry(
            "  --very-long-flag-name",
            "Description", 
            15, 
            60
        );
        // Should put description on next line when left column is too long
        assert!(result.contains('\n'));
    }

    #[test]
    fn test_wrap_text_preserves_empty_lines() {
        let text = "First paragraph\n\nSecond paragraph";
        let wrapped = wrap_text(text, 50, None);
        assert_eq!(wrapped, "First paragraph\n\nSecond paragraph");
    }
}