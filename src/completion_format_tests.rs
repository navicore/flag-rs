#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::CompletionResult;
    use crate::completion_format::CompletionFormat;

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
        assert!(formatted[0].contains(":"));
        assert!(formatted[0].contains(" - "));
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
            assert!(line.contains(":"));
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
        // Values should be properly formatted
        assert_eq!(formatted.len(), 3);
    }

    #[test]
    fn test_fish_format_empty_description() {
        let result = CompletionResult::new()
            .add("no-desc-value")
            .add_with_description("with-desc", "Description");

        let formatted = CompletionFormat::Fish.format(&result);

        // Fish can have values without descriptions
        assert_eq!(formatted[0], "no-desc-value");
        assert!(formatted[1].contains("\t"));
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
        let long_value = "a".repeat(100);
        let long_desc = "b".repeat(100);

        let result = CompletionResult::new()
            .add(&long_value)
            .add_with_description(&long_value, &long_desc);

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
