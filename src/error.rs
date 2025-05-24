use std::fmt;

#[derive(Debug)]
pub enum Error {
    CommandNotFound(String),
    SubcommandRequired(String),
    NoRunFunction(String),
    FlagParsing(String),
    ArgumentParsing(String),
    Validation(String),
    Completion(String),
    Io(std::io::Error),
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
