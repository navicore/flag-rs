//! Dynamic shell completion support
//!
//! This module provides the infrastructure for dynamic completions that are
//! computed at runtime when the user presses TAB, rather than being hardcoded
//! at compile time.

use crate::active_help::ActiveHelp;
use crate::context::Context;
use crate::error::Result;

/// Result returned by completion functions
///
/// `CompletionResult` contains completion suggestions along with optional
/// descriptions for each suggestion. This is used by the shell completion
/// system to provide helpful hints to users.
///
/// # Examples
///
/// ```
/// use flag_rs::completion::CompletionResult;
///
/// let completions = CompletionResult::new()
///     .add("create")
///     .add_with_description("delete", "Remove a resource")
///     .add_with_description("list", "Show all resources")
///     .extend(vec!["get".to_string(), "update".to_string()]);
///
/// assert_eq!(completions.values.len(), 5);
/// assert_eq!(completions.values[1], "delete");
/// assert_eq!(completions.descriptions[1], "Remove a resource");
/// ```
#[derive(Clone, Debug)]
pub struct CompletionResult {
    /// The completion values to suggest
    pub values: Vec<String>,
    /// Optional descriptions for each value
    pub descriptions: Vec<String>,
    /// `ActiveHelp` messages to display
    pub active_help: Vec<ActiveHelp>,
}

impl CompletionResult {
    /// Creates a new empty completion result
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            descriptions: Vec::new(),
            active_help: Vec::new(),
        }
    }

    /// Adds a completion value without a description
    ///
    /// # Arguments
    ///
    /// * `value` - The completion value to add
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::completion::CompletionResult;
    ///
    /// let result = CompletionResult::new()
    ///     .add("option1")
    ///     .add("option2");
    /// ```
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add(mut self, value: impl Into<String>) -> Self {
        self.values.push(value.into());
        self.descriptions.push(String::new());
        self
    }

    /// Adds a completion value with a description
    ///
    /// # Arguments
    ///
    /// * `value` - The completion value to add
    /// * `desc` - A description of what this value represents
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::completion::CompletionResult;
    ///
    /// let result = CompletionResult::new()
    ///     .add_with_description("--verbose", "Enable verbose output")
    ///     .add_with_description("--quiet", "Suppress output");
    /// ```
    #[must_use]
    pub fn add_with_description(
        mut self,
        value: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        self.values.push(value.into());
        self.descriptions.push(desc.into());
        self
    }

    /// Adds multiple completion values without descriptions
    ///
    /// # Arguments
    ///
    /// * `values` - An iterator of completion values
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::completion::CompletionResult;
    ///
    /// let options = vec!["opt1".to_string(), "opt2".to_string()];
    /// let result = CompletionResult::new().extend(options);
    /// ```
    #[must_use]
    pub fn extend<I: IntoIterator<Item = String>>(mut self, values: I) -> Self {
        for value in values {
            self.values.push(value);
            self.descriptions.push(String::new());
        }
        self
    }

    /// Adds an `ActiveHelp` message
    ///
    /// # Arguments
    ///
    /// * `help` - The `ActiveHelp` message to add
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::completion::CompletionResult;
    /// use flag_rs::active_help::ActiveHelp;
    ///
    /// let result = CompletionResult::new()
    ///     .add_help(ActiveHelp::new("Press TAB to see available options"));
    /// ```
    #[must_use]
    pub fn add_help(mut self, help: ActiveHelp) -> Self {
        self.active_help.push(help);
        self
    }

    /// Adds an `ActiveHelp` message from a string
    ///
    /// # Arguments
    ///
    /// * `message` - The help message text
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::completion::CompletionResult;
    ///
    /// let result = CompletionResult::new()
    ///     .add_help_text("Use -n <namespace> to filter results");
    /// ```
    #[must_use]
    pub fn add_help_text<S: Into<String>>(mut self, message: S) -> Self {
        self.active_help.push(ActiveHelp::new(message));
        self
    }

    /// Adds a conditional `ActiveHelp` message
    ///
    /// # Arguments
    ///
    /// * `message` - The help message text
    /// * `condition` - Function that determines if help should be shown
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::completion::CompletionResult;
    ///
    /// let result = CompletionResult::new()
    ///     .add_conditional_help(
    ///         "Tip: Use --format json for machine-readable output",
    ///         |ctx| ctx.flag("format").is_none()
    ///     );
    /// ```
    #[must_use]
    pub fn add_conditional_help<S, F>(mut self, message: S, condition: F) -> Self
    where
        S: Into<String>,
        F: Fn(&Context) -> bool + Send + Sync + 'static,
    {
        self.active_help
            .push(ActiveHelp::with_condition(message, condition));
        self
    }
}

impl Default for CompletionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for completion functions
///
/// Completion functions are called when the user presses TAB to get suggestions.
/// They receive the current context and the partial text being completed.
///
/// # Arguments
///
/// * `&Context` - The current command context with flags and arguments
/// * `&str` - The partial text being completed
///
/// # Returns
///
/// Returns a `Result<CompletionResult>` with the suggested completions
///
/// # Examples
///
/// ```
/// use flag_rs::completion::{CompletionFunc, CompletionResult};
/// use flag_rs::context::Context;
/// use flag_rs::error::Result;
///
/// // A completion function that suggests file names
/// let file_completer: CompletionFunc = Box::new(|_ctx, partial| {
///     Ok(CompletionResult::new()
///         .add("file1.txt")
///         .add("file2.txt")
///         .add("file3.log"))
/// });
///
/// // A dynamic completion function that uses context
/// let pod_completer: CompletionFunc = Box::new(|ctx, partial| {
///     // In a real implementation, this would query the Kubernetes API
///     let namespace = ctx.flag("namespace")
///         .map(|s| s.as_str())
///         .unwrap_or("default");
///     Ok(CompletionResult::new()
///         .add_with_description("pod-abc-123", format!("Pod in namespace {}", namespace))
///         .add_with_description("pod-def-456", format!("Pod in namespace {}", namespace)))
/// });
/// ```
pub type CompletionFunc = Box<dyn Fn(&Context, &str) -> Result<CompletionResult> + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_result() {
        let result = CompletionResult::new()
            .add("option1")
            .add_with_description("option2", "Description for option2")
            .extend(vec!["option3".to_string(), "option4".to_string()]);

        assert_eq!(result.values.len(), 4);
        assert_eq!(result.descriptions.len(), 4);

        assert_eq!(result.values[0], "option1");
        assert_eq!(result.descriptions[0], "");

        assert_eq!(result.values[1], "option2");
        assert_eq!(result.descriptions[1], "Description for option2");

        assert_eq!(result.values[2], "option3");
        assert_eq!(result.descriptions[2], "");
    }

    #[test]
    fn test_completion_result_with_active_help() {
        let result = CompletionResult::new()
            .add("option1")
            .add_help_text("This is a help message")
            .add_conditional_help("Conditional help", |_| true);

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.active_help.len(), 2);
        assert_eq!(result.active_help[0].message, "This is a help message");
        assert_eq!(result.active_help[1].message, "Conditional help");
    }
}
