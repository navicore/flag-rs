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
    /// Contains the name of the unknown command.
    CommandNotFound(String),

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
    /// (e.g., "abc" for an integer flag). Contains a description of the error.
    FlagParsing(String),

    /// An error occurred while parsing command arguments
    ///
    /// This error occurs when command arguments don't meet requirements.
    /// Contains a description of the error.
    ArgumentParsing(String),

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
        match self {
            Self::CommandNotFound(cmd) => write!(f, "Unknown command: {cmd}"),
            Self::SubcommandRequired(cmd) => write!(f, "'{cmd}' requires a subcommand"),
            Self::NoRunFunction(cmd) => write!(f, "Command '{cmd}' is not runnable"),
            Self::FlagParsing(msg) => write!(f, "Invalid flag: {msg}"),
            Self::ArgumentParsing(msg) => write!(f, "Invalid argument: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::Completion(msg) => write!(f, "Completion error: {msg}"),
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Custom(err) => write!(f, "Error: {err}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::CommandNotFound("test".to_string()).to_string(),
            "Unknown command: test"
        );
        assert_eq!(
            Error::SubcommandRequired("kubectl".to_string()).to_string(),
            "'kubectl' requires a subcommand"
        );
        assert_eq!(
            Error::FlagParsing("--invalid".to_string()).to_string(),
            "Invalid flag: --invalid"
        );
    }

    #[test]
    fn test_error_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::Io(io_error);
        assert!(error.source().is_some());

        let error = Error::CommandNotFound("test".to_string());
        assert!(error.source().is_none());
    }
}
