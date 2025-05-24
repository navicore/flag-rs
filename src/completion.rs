//! Dynamic shell completion support
//!
//! This module provides the infrastructure for dynamic completions that are
//! computed at runtime when the user presses TAB, rather than being hardcoded
//! at compile time.

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
}

impl CompletionResult {
    /// Creates a new empty completion result
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            descriptions: Vec::new(),
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
}
