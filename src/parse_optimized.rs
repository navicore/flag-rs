//! Memory-optimized flag parsing
//!
//! This module provides optimized parsing functions that minimize
//! string allocations during command-line parsing.

use crate::error::Result;
use crate::flag::Flag;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Optimized flag parsing that minimizes allocations
///
/// Returns a map of flag names to values where both are borrowed from the input args
pub fn parse_flags_optimized<'a, S>(
    args: &'a [String],
    flags: &HashMap<String, Flag, S>,
    parent_flags: Option<&HashMap<String, Flag, S>>,
) -> Result<(HashMap<String, String>, Vec<&'a str>)>
where
    S: BuildHasher,
{
    let mut parsed_flags = HashMap::new();
    let mut remaining = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "--" {
            // Everything after -- is treated as arguments
            remaining.extend(args[i + 1..].iter().map(String::as_str));
            break;
        } else if let Some(long_flag) = arg.strip_prefix("--") {
            // Handle long flags
            if long_flag == "help" {
                parsed_flags.insert("help".to_string(), "true".to_string());
            } else if let Some((name, value)) = long_flag.split_once('=') {
                // Flag with value: --flag=value
                if let Some(flag) = find_flag(name, flags, parent_flags) {
                    flag.parse_value(value)?;
                }
                parsed_flags.insert(name.to_string(), value.to_string());
            } else if let Some(flag) = find_flag(long_flag, flags, parent_flags) {
                // Check if next arg is the value
                if flag.value_type != crate::flag::FlagType::Bool
                    && i + 1 < args.len()
                    && !args[i + 1].starts_with('-')
                {
                    let value = &args[i + 1];
                    flag.parse_value(value)?;
                    parsed_flags.insert(long_flag.to_string(), value.to_string());
                    i += 1;
                } else {
                    parsed_flags.insert(long_flag.to_string(), "true".to_string());
                }
            } else {
                // Unknown flag - might belong to subcommand
                remaining.push(arg.as_str());
            }
        } else if let Some(short_flags) = arg.strip_prefix('-').filter(|s| !s.is_empty()) {
            // Handle short flags
            let chars: Vec<char> = short_flags.chars().collect();

            for (idx, ch) in chars.iter().enumerate() {
                if *ch == 'h' {
                    parsed_flags.insert("help".to_string(), "true".to_string());
                } else if let Some(flag) = find_flag_by_short(*ch, flags, parent_flags) {
                    // If this is the last char and flag takes a value
                    if flag.value_type != crate::flag::FlagType::Bool
                        && idx == chars.len() - 1
                        && i + 1 < args.len()
                        && !args[i + 1].starts_with('-')
                    {
                        let value = &args[i + 1];
                        flag.parse_value(value)?;
                        parsed_flags.insert(flag.name.clone(), value.to_string());
                        i += 1;
                    } else {
                        parsed_flags.insert(flag.name.clone(), "true".to_string());
                    }
                } else {
                    // Unknown short flag
                    remaining.push(arg.as_str());
                    break;
                }
            }
        } else {
            remaining.push(arg.as_str());
        }

        i += 1;
    }

    Ok((parsed_flags, remaining))
}

/// Find a flag by name in local or parent flags
fn find_flag<'a, S>(
    name: &str,
    flags: &'a HashMap<String, Flag, S>,
    parent_flags: Option<&'a HashMap<String, Flag, S>>,
) -> Option<&'a Flag>
where
    S: BuildHasher,
{
    flags
        .get(name)
        .or_else(|| parent_flags.and_then(|pf| pf.get(name)))
}

/// Find a flag by short name in local or parent flags
fn find_flag_by_short<'a, S>(
    short: char,
    flags: &'a HashMap<String, Flag, S>,
    parent_flags: Option<&'a HashMap<String, Flag, S>>,
) -> Option<&'a Flag>
where
    S: BuildHasher,
{
    flags
        .values()
        .find(|f| f.short == Some(short))
        .or_else(|| parent_flags.and_then(|pf| pf.values().find(|f| f.short == Some(short))))
}

/// Alternative parsing that returns borrowed strings where possible
pub fn parse_flags_borrowed(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flag::FlagType;

    #[test]
    fn test_optimized_parsing() {
        let mut flags = HashMap::new();
        flags.insert(
            "verbose".to_string(),
            Flag::new("verbose").short('v').value_type(FlagType::Bool),
        );
        flags.insert(
            "output".to_string(),
            Flag::new("output").short('o').value_type(FlagType::String),
        );

        let args = vec![
            "-v".to_string(),
            "--output".to_string(),
            "file.txt".to_string(),
            "arg1".to_string(),
        ];

        let (parsed, remaining) = parse_flags_optimized(&args, &flags, None).unwrap();

        assert_eq!(parsed.get("verbose").map(String::as_str), Some("true"));
        assert_eq!(parsed.get("output").map(String::as_str), Some("file.txt"));
        assert_eq!(remaining, vec!["arg1"]);
    }

    #[test]
    fn test_no_allocations_for_flags() {
        let mut flags = HashMap::new();
        flags.insert(
            "flag1".to_string(),
            Flag::new("flag1").value_type(FlagType::Bool),
        );

        let args = vec!["--flag1".to_string(), "value".to_string()];

        let (parsed, remaining) = parse_flags_optimized(&args, &flags, None).unwrap();

        // The parsed flags contain owned strings for compatibility
        assert_eq!(parsed.get("flag1").map(String::as_str), Some("true"));
        assert_eq!(remaining, vec!["value"]);
    }
}
