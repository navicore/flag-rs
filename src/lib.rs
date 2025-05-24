pub mod color;
pub mod command;
pub mod completion;
pub mod context;
pub mod error;
pub mod flag;
pub mod shell;

pub use command::{Command, CommandBuilder};
pub use completion::{CompletionFunc, CompletionResult};
pub use context::Context;
pub use error::{Error, Result};
pub use flag::{Flag, FlagType, FlagValue};
pub use shell::Shell;
