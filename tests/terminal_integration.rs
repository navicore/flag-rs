//! Integration tests for terminal functionality
//!
//! These tests validate terminal width detection and text wrapping
//! in a way that's maintainable and protects against regressions.

use flag_rs::terminal::{format_help_entry, get_terminal_width, wrap_text};
use std::env;

#[test]
fn test_terminal_width_respects_columns_env() {
    // Save original COLUMNS value
    let original = env::var("COLUMNS").ok();

    // Test various COLUMNS values
    let test_cases = vec![
        ("50", 50),   // Below default
        ("80", 80),   // Default
        ("100", 100), // Above default
        ("30", 40),   // Below minimum (should clamp to 40)
        ("200", 120), // Above maximum (should clamp to 120)
    ];

    for (env_value, expected) in test_cases {
        env::set_var("COLUMNS", env_value);
        assert_eq!(
            get_terminal_width(),
            expected,
            "COLUMNS={env_value} should result in width={expected}"
        );
    }

    // Test with invalid COLUMNS
    env::set_var("COLUMNS", "invalid");
    assert_eq!(
        get_terminal_width(),
        80,
        "Invalid COLUMNS should fall back to 80"
    );

    // Restore original
    match original {
        Some(val) => env::set_var("COLUMNS", val),
        None => env::remove_var("COLUMNS"),
    }
}

#[test]
fn test_wrap_text_preserves_content() {
    let text = "The quick brown fox jumps over the lazy dog";
    let wrapped = wrap_text(text, 20, None);

    // Ensure all words are preserved
    let original_words: Vec<&str> = text.split_whitespace().collect();
    let wrapped_words: Vec<&str> = wrapped.split_whitespace().collect();
    assert_eq!(
        original_words, wrapped_words,
        "Wrapping should preserve all words"
    );
}

#[test]
fn test_wrap_text_respects_width() {
    let text = "This is a very long line that needs to be wrapped at specific boundaries";
    let width = 20;
    let wrapped = wrap_text(text, width, None);

    for line in wrapped.lines() {
        assert!(
            line.len() <= width,
            "Line '{}' exceeds width {} (actual: {})",
            line,
            width,
            line.len()
        );
    }
}

#[test]
fn test_wrap_text_handles_indentation() {
    let text = "First line should not be indented but subsequent lines should have indent";
    let wrapped = wrap_text(text, 30, Some(4));
    let lines: Vec<&str> = wrapped.lines().collect();

    assert!(lines.len() > 1, "Text should wrap to multiple lines");
    assert!(
        !lines[0].starts_with("    "),
        "First line should not be indented"
    );

    for line in &lines[1..] {
        assert!(
            line.starts_with("    "),
            "Continuation line '{line}' should be indented"
        );
    }
}

#[test]
fn test_format_help_entry_alignment() {
    // Normal case - left column fits
    let result = format_help_entry("  -v, --verbose", "Enable verbose output", 20, 60);

    // Should have proper spacing between columns
    assert!(result.contains("  -v, --verbose"));
    assert!(result.contains("Enable verbose output"));

    // The description should start at position 22 (20 + 2 spaces)
    let first_line = result.lines().next().unwrap();
    let desc_start = first_line.find("Enable").unwrap();
    assert_eq!(desc_start, 22, "Description should be properly aligned");
}

#[test]
fn test_format_help_entry_long_flag() {
    // Case where left column is too long
    let result = format_help_entry(
        "  --very-long-flag-name-that-exceeds-column",
        "Description goes here",
        20,
        60,
    );

    // Should put description on next line
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Long flag should push description to next line"
    );
    assert!(lines[0].contains("--very-long-flag-name"));
    assert!(lines[1].trim().starts_with("Description"));
}

#[test]
fn test_wrap_text_empty_and_edge_cases() {
    // Empty text
    assert_eq!(wrap_text("", 80, None), "");

    // Single word longer than width
    let long_word = "supercalifragilisticexpialidocious";
    let wrapped = wrap_text(long_word, 10, None);
    assert_eq!(wrapped, long_word, "Long words should not be broken");

    // Text with multiple spaces
    let spaced = "word1     word2     word3";
    let wrapped = wrap_text(spaced, 80, None);
    assert_eq!(
        wrapped, "word1 word2 word3",
        "Multiple spaces should be normalized"
    );
}

#[test]
fn test_terminal_width_snapshot() {
    // This test creates a "snapshot" of expected behavior
    // that makes it easy to spot regressions
    let test_cases = vec![
        ("Short text", 80, "Short text"),
        (
            "This is a medium length text that might wrap",
            20,
            "This is a medium\nlength text that\nmight wrap",
        ),
        (
            "Text with\nmultiple\nparagraphs",
            80,
            "Text with\nmultiple\nparagraphs",
        ),
    ];

    for (input, width, expected) in test_cases {
        let result = wrap_text(input, width, None);
        assert_eq!(
            result, expected,
            "\nInput: '{input}'\nWidth: {width}\nExpected: '{expected}'\nActual: '{result}'"
        );
    }
}

/// Helper to create a visual test output for manual verification
#[test]
#[ignore] // Run with: cargo test -- --ignored --nocapture
fn visual_terminal_test() {
    println!("\n=== Terminal Width Detection ===");
    println!("Current terminal width: {}", get_terminal_width());
    println!("COLUMNS env var: {:?}", env::var("COLUMNS"));

    println!("\n=== Text Wrapping Examples ===");
    let widths = vec![40, 60, 80, 100];
    let sample_text = "This is a sample text that demonstrates the text wrapping functionality. It should wrap nicely at word boundaries and respect the specified width.";

    for width in widths {
        println!("\nWidth {width}:");
        println!("{}", "-".repeat(width));
        println!("{}", wrap_text(sample_text, width, None));
        println!("{}", "-".repeat(width));
    }

    println!("\n=== Help Entry Formatting ===");
    let flags = vec![
        ("  -h, --help", "Show help information"),
        (
            "  -v, --verbose",
            "Enable verbose output with detailed logging",
        ),
        (
            "  --very-long-flag-name",
            "This flag has a long name that might affect formatting",
        ),
    ];

    for (flag, desc) in flags {
        println!("{}", format_help_entry(flag, desc, 25, 80));
    }
}
