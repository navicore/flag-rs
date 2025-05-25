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
    pub fn format(self, result: &CompletionResult) -> Vec<String> {
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
        let max_width = result.values.iter().map(String::len).max().unwrap_or(0);
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
        // Terminal width constraint
        const MAX_WIDTH: usize = 80;

        // Calculate max width for alignment, but cap it
        let max_value_width = result.values.iter().map(String::len).max().unwrap_or(0);
        // Limit padding to ensure we don't exceed terminal width
        // Reserve space for ": - " (4 chars) and some description text
        let padding = max_value_width.min(35) + 4;

        result
            .values
            .iter()
            .zip(&result.descriptions)
            .map(|(value, desc)| {
                // We need to escape colons in the value
                let escaped_value = value.replace(':', "\\:");

                if desc.is_empty() {
                    // Even without description, use the standard format
                    // Match the working format: value:value    - description
                    format!("{escaped_value}:{escaped_value}    - ")
                } else {
                    // Zsh format: value:description
                    // Format with padding
                    let formatted_desc = if value.len() <= 35 {
                        format!("{escaped_value:<padding$}- {desc}")
                    } else {
                        // For very long values, skip padding
                        format!("{escaped_value} - {desc}")
                    };

                    // Truncate if still too long
                    let full_line = format!("{escaped_value}:{formatted_desc}");
                    if full_line.len() > MAX_WIDTH {
                        format!("{}...", &full_line[..MAX_WIDTH - 3])
                    } else {
                        full_line
                    }
                }
            })
            .collect()
    }

    /// Formats for Fish completion
    fn format_fish(result: &CompletionResult) -> Vec<String> {
        // Terminal width constraint
        const MAX_WIDTH: usize = 80;

        // Calculate max width for alignment, but cap it
        let max_value_width = result.values.iter().map(String::len).max().unwrap_or(0);
        // Limit padding to ensure we don't exceed terminal width
        let padding = max_value_width.min(35) + 4;

        result
            .values
            .iter()
            .zip(&result.descriptions)
            .map(|(value, desc)| {
                if desc.is_empty() {
                    // For fish, just the value is fine without description
                    value.clone()
                } else {
                    // Fish format: value\tdescription
                    // Format with padding
                    let formatted_desc = if value.len() <= 35 {
                        format!("{value:<padding$}- {desc}")
                    } else {
                        // For very long values, skip padding
                        format!("{value} - {desc}")
                    };

                    // Fish uses tab separation, but still check total length
                    let full_line = format!("{value}\t{formatted_desc}");
                    if formatted_desc.len() > MAX_WIDTH {
                        let truncated_desc = format!("{}...", &formatted_desc[..MAX_WIDTH - 3]);
                        format!("{value}\t{truncated_desc}")
                    } else {
                        full_line
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::CompletionResult;

    #[test]
    fn test_zsh_format_with_empty_description() {
        let result = CompletionResult::new()
            .add("value-without-desc")
            .add_with_description("value-with-desc", "This has a description");

        let formatted = CompletionFormat::Zsh.format(&result);

        // Empty descriptions should still produce proper zsh format
        assert_eq!(formatted.len(), 2);
        assert!(formatted[0].starts_with("value-without-desc:"));
        assert!(formatted[0].contains(" - "));
        assert!(formatted[1].starts_with("value-with-desc:"));
    }

    #[test]
    fn test_zsh_format_uuid_without_description() {
        // Test case that caused the invisible completion bug
        let result = CompletionResult::new().add("28cbc1d1-7750-4253-9f55-ae21b9156b9d");

        let formatted = CompletionFormat::Zsh.format(&result);

        assert_eq!(formatted.len(), 1);
        // Must have the zsh format even without description
        assert!(formatted[0].contains(':'));
        assert!(formatted[0].contains(" - "));
        // Check exact format
        assert_eq!(
            formatted[0],
            "28cbc1d1-7750-4253-9f55-ae21b9156b9d:28cbc1d1-7750-4253-9f55-ae21b9156b9d    - "
        );
    }

    #[test]
    fn test_empty_value_handling() {
        let result = CompletionResult::new()
            .add("")
            .add_with_description("", "Empty value with description");

        let formatted = CompletionFormat::Zsh.format(&result);

        // Even empty values should be formatted properly
        assert_eq!(formatted.len(), 2);
        for line in &formatted {
            assert!(line.contains(':'));
        }
    }

    #[test]
    fn test_special_characters_in_value() {
        let result = CompletionResult::new()
            .add("value:with:colons")
            .add("value'with'quotes")
            .add("value with spaces");

        let formatted = CompletionFormat::Zsh.format(&result);

        // Colons should be escaped
        assert!(formatted[0].starts_with("value\\:with\\:colons:"));
        // All values should be properly formatted
        assert_eq!(formatted.len(), 3);
        for line in &formatted {
            assert!(line.contains(" - "));
        }
    }

    #[test]
    fn test_fish_format_empty_description() {
        let result = CompletionResult::new()
            .add("no-desc-value")
            .add_with_description("with-desc", "Description");

        let formatted = CompletionFormat::Fish.format(&result);

        // Fish can have values without descriptions
        assert_eq!(formatted[0], "no-desc-value");
        assert!(formatted[1].contains('\t'));
    }

    #[test]
    fn test_bash_format() {
        let result = CompletionResult::new()
            .add("value1")
            .add_with_description("value2", "Description ignored for bash");

        let formatted = CompletionFormat::Bash.format(&result);

        // Bash format is just the values
        assert_eq!(formatted, vec!["value1", "value2"]);
    }

    #[test]
    fn test_line_length_limits() {
        let long_value = "a".repeat(50);
        let long_desc = "b".repeat(50);

        let result = CompletionResult::new().add_with_description(&long_value, &long_desc);

        let formatted = CompletionFormat::Zsh.format(&result);

        // All lines should be <= 80 characters
        for line in formatted {
            assert!(line.len() <= 80, "Line too long: {} chars", line.len());
            if line.len() == 80 {
                assert!(
                    line.ends_with("..."),
                    "Long lines should be truncated with ..."
                );
            }
        }
    }
}
