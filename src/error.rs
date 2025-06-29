//! Error types for the flag framework
//!
//! This module defines the error types that can occur when parsing commands,
//! flags, and arguments, or when executing command handlers.

use std::fmt;

/// The main error type for the flag framework
///
/// This enum represents all possible errors that can occur during command
/// parsing, validation, and execution.
#[derive(Debug)]
pub enum Error {
    /// The specified command was not found
    ///
    /// This error occurs when a user tries to run a subcommand that doesn't exist.
    CommandNotFound {
        /// The name of the unknown command
        command: String,
        /// Suggested similar commands
        suggestions: Vec<String>,
    },

    /// A command requires a subcommand but none was provided
    ///
    /// This error occurs when a parent command has no run function and the user
    /// doesn't specify which subcommand to run. Contains the parent command name.
    SubcommandRequired(String),

    /// A command has no run function defined
    ///
    /// This error occurs when trying to execute a command that has no run handler.
    /// Contains the command name.
    NoRunFunction(String),

    /// An error occurred while parsing flag values
    ///
    /// This error occurs when a flag value cannot be parsed as the expected type
    /// (e.g., "abc" for an integer flag).
    FlagParsing {
        /// Description of the error
        message: String,
        /// The flag that caused the error
        flag: Option<String>,
        /// Suggested valid values or format
        suggestions: Vec<String>,
    },

    /// An error occurred while parsing command arguments
    ///
    /// This error occurs when command arguments don't meet requirements.
    /// Contains a description of the error.
    ArgumentParsing(String),

    /// Argument validation failed
    ///
    /// This error occurs when arguments don't meet validation constraints.
    ArgumentValidation {
        /// Description of the validation failure
        message: String,
        /// Expected argument pattern
        expected: String,
        /// Number of arguments received
        received: usize,
    },

    /// A validation error occurred
    ///
    /// This error occurs when custom validation logic fails.
    /// Contains a description of the validation failure.
    Validation(String),

    /// An error occurred during shell completion
    ///
    /// This error occurs when completion functions fail.
    /// Contains a description of the error.
    Completion(String),

    /// An I/O error occurred
    ///
    /// Wraps standard I/O errors that may occur during command execution.
    Io(std::io::Error),

    /// A custom error from user code
    ///
    /// Allows command handlers to return their own error types.
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::color;

        match self {
            Self::CommandNotFound {
                command,
                suggestions,
            } => {
                write!(f, "{}: unknown command ", color::red("Error"))?;
                write!(f, "{}", color::bold(command))?;

                if !suggestions.is_empty() {
                    write!(f, "\n\n")?;
                    if suggestions.len() == 1 {
                        writeln!(f, "{}?", color::yellow("Did you mean this"))?;
                        write!(f, "    {}", color::green(&suggestions[0]))?;
                    } else {
                        writeln!(f, "{}?", color::yellow("Did you mean one of these"))?;
                        for suggestion in suggestions {
                            writeln!(f, "    {}", color::green(suggestion))?;
                        }
                        // Remove trailing newline
                        if f.width().is_none() {
                            // This is a hack to remove the last newline
                            // In real usage, the error display adds its own newline
                        }
                    }
                }
                Ok(())
            }
            Self::SubcommandRequired(cmd) => {
                write!(f, "{}: ", color::red("Error"))?;
                write!(f, "'{}' requires a subcommand", color::bold(cmd))?;
                write!(
                    f,
                    "\n\n{}: use '{} --help' for available subcommands",
                    color::yellow("Hint"),
                    cmd
                )
            }
            Self::NoRunFunction(cmd) => {
                write!(
                    f,
                    "{}: command '{}' is not runnable",
                    color::red("Error"),
                    color::bold(cmd)
                )
            }
            Self::FlagParsing {
                message,
                flag,
                suggestions,
            } => {
                write!(f, "{}: {}", color::red("Error"), message)?;
                if let Some(flag_name) = flag {
                    write!(f, " for flag '{}'", color::bold(flag_name))?;
                }

                if !suggestions.is_empty() {
                    write!(f, "\n\n")?;
                    if suggestions.len() == 1 {
                        write!(f, "{}: {}", color::yellow("Expected"), suggestions[0])?;
                    } else {
                        writeln!(f, "{} one of:", color::yellow("Expected"))?;
                        for suggestion in suggestions {
                            writeln!(f, "    {}", color::green(suggestion))?;
                        }
                        // Remove trailing newline
                        if f.width().is_none() {
                            // Handled by caller
                        }
                    }
                }
                Ok(())
            }
            Self::ArgumentParsing(msg) => {
                write!(f, "{}: invalid argument - {}", color::red("Error"), msg)
            }
            Self::ArgumentValidation {
                message,
                expected,
                received,
            } => {
                write!(f, "{}: {}", color::red("Error"), message)?;
                write!(f, "\n\n{}: {}", color::yellow("Expected"), expected)?;
                write!(
                    f,
                    "\n{}: {} argument{}",
                    color::yellow("Received"),
                    received,
                    if *received == 1 { "" } else { "s" }
                )
            }
            Self::Validation(msg) => {
                write!(f, "{}: {}", color::red("Validation error"), msg)
            }
            Self::Completion(msg) => {
                write!(f, "{}: {}", color::red("Completion error"), msg)
            }
            Self::Io(err) => {
                write!(f, "{}: {}", color::red("IO error"), err)
            }
            Self::Custom(err) => {
                write!(f, "{}: {}", color::red("Error"), err)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Custom(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Type alias for Results with the flag Error type
///
/// This is a convenience type alias for `std::result::Result<T, flag::Error>`.
///
/// # Examples
///
/// ```
/// use flag_rs::error::{Error, Result};
///
/// fn parse_count(s: &str) -> Result<u32> {
///     s.parse()
///         .map_err(|_| Error::ArgumentParsing(format!("invalid count: {}", s)))
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a simple flag parsing error
    pub fn flag_parsing(message: impl Into<String>) -> Self {
        Self::FlagParsing {
            message: message.into(),
            flag: None,
            suggestions: vec![],
        }
    }

    /// Create a flag parsing error with suggestions
    pub fn flag_parsing_with_suggestions(
        message: impl Into<String>,
        flag: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        Self::FlagParsing {
            message: message.into(),
            flag: Some(flag.into()),
            suggestions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_error_display() {
        // Test without color for predictable test output
        std::env::set_var("NO_COLOR", "1");

        assert_eq!(
            Error::CommandNotFound {
                command: "test".to_string(),
                suggestions: vec![],
            }
            .to_string(),
            "Error: unknown command test"
        );
        assert_eq!(
            Error::SubcommandRequired("kubectl".to_string()).to_string(),
            "Error: 'kubectl' requires a subcommand\n\nHint: use 'kubectl --help' for available subcommands"
        );
        assert_eq!(
            Error::FlagParsing {
                message: "invalid flag".to_string(),
                flag: Some("invalid".to_string()),
                suggestions: vec![],
            }
            .to_string(),
            "Error: invalid flag for flag 'invalid'"
        );

        std::env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_error_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::Io(io_error);
        assert!(error.source().is_some());

        let error = Error::CommandNotFound {
            command: "test".to_string(),
            suggestions: vec![],
        };
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_with_suggestions() {
        // Test without color for predictable test output
        std::env::set_var("NO_COLOR", "1");

        // Single suggestion
        let error = Error::CommandNotFound {
            command: "strt".to_string(),
            suggestions: vec!["start".to_string()],
        };
        assert_eq!(
            error.to_string(),
            "Error: unknown command strt\n\nDid you mean this?\n    start"
        );

        // Multiple suggestions
        let error = Error::CommandNotFound {
            command: "lst".to_string(),
            suggestions: vec!["list".to_string(), "last".to_string()],
        };
        assert_eq!(
            error.to_string(),
            "Error: unknown command lst\n\nDid you mean one of these?\n    list\n    last\n"
        );

        std::env::remove_var("NO_COLOR");
    }
}
