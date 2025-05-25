//! Completion format handling for different shells
//!
//! This module defines how completions are formatted for different shells,
//! including support for descriptions where the shell supports them.

use crate::completion::CompletionResult;

/// Represents the format in which completions should be returned
#[derive(Debug, Clone, Copy)]
pub enum CompletionFormat {
    /// Simple list of values (for basic shells)
    Simple,
    /// Values with descriptions for display (not for shell consumption)
    Display,
    /// Zsh format with descriptions
    Zsh,
    /// Fish format with descriptions
    Fish,
    /// Bash format (requires special handling)
    Bash,
}

impl CompletionFormat {
    /// Detects the format from the shell type string
    pub fn from_shell_type(shell_type: Option<&str>) -> Self {
        match shell_type {
            Some("zsh") => Self::Zsh,
            Some("fish") => Self::Fish,
            Some("bash") => Self::Bash,
            Some("display") => Self::Display,
            _ => Self::Simple,
        }
    }

    /// Formats a completion result according to this format
    pub fn format(&self, result: &CompletionResult) -> Vec<String> {
        match self {
            Self::Simple | Self::Bash => {
                // For bash and simple format, return just the values
                result.values.clone()
            }
            Self::Display => {
                // For display, show formatted with descriptions
                Self::format_display(result)
            }
            Self::Zsh => {
                // Zsh has special syntax for descriptions
                Self::format_zsh(result)
            }
            Self::Fish => {
                // Fish uses tab-separated format
                Self::format_fish(result)
            }
        }
    }

    /// Formats for human-readable display (not shell consumption)
    fn format_display(result: &CompletionResult) -> Vec<String> {
        use crate::color;

        let has_descriptions = result.descriptions.iter().any(|d| !d.is_empty());
        if !has_descriptions {
            return result.values.clone();
        }

        // Calculate column width
        let max_width = result.values.iter().map(|v| v.len()).max().unwrap_or(0);
        let column_width = max_width + 4;

        result
            .values
            .iter()
            .zip(&result.descriptions)
            .map(|(value, desc)| {
                if desc.is_empty() {
                    value.clone()
                } else {
                    let padded = format!("{value:<column_width$}");
                    if color::should_colorize() {
                        format!("{padded}{}", color::dim(desc))
                    } else {
                        format!("{padded}{desc}")
                    }
                }
            })
            .collect()
    }

    /// Formats for Zsh completion
    fn format_zsh(result: &CompletionResult) -> Vec<String> {
        
        // Calculate max width for alignment
        let max_width = result.values.iter().map(|v| v.len()).max().unwrap_or(0);
        let padding = max_width + 4; // Add some extra padding
        
        
        result
            .values
            .iter()
            .zip(&result.descriptions)
            .map(|(value, desc)| {
                if desc.is_empty() {
                    value.clone()
                } else {
                    // Zsh format: value:description
                    // We need to escape colons in the value
                    let escaped_value = value.replace(':', "\\:");
                    // Include the value in the description with padding for alignment
                    // Note: zsh doesn't support ANSI colors in completion descriptions
                    let formatted_desc = format!("{value:<padding$}- {desc}");
                    format!("{escaped_value}:{formatted_desc}")
                }
            })
            .collect()
    }

    /// Formats for Fish completion
    fn format_fish(result: &CompletionResult) -> Vec<String> {
        
        // Calculate max width for alignment
        let max_width = result.values.iter().map(|v| v.len()).max().unwrap_or(0);
        let padding = max_width + 4; // Add some extra padding
        
        
        result
            .values
            .iter()
            .zip(&result.descriptions)
            .map(|(value, desc)| {
                if desc.is_empty() {
                    value.clone()
                } else {
                    // Fish format: value\tdescription
                    // Include the value in the description with padding for alignment
                    // Note: zsh doesn't support ANSI colors in completion descriptions
                    let formatted_desc = format!("{value:<padding$}- {desc}");
                    format!("{value}\t{formatted_desc}")
                }
            })
            .collect()
    }
}
