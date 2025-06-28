//! Memory-optimized completion result structure
//!
//! This module provides a memory-efficient alternative to CompletionResult
//! that reduces allocations and memory usage for large CLIs.

use crate::active_help::ActiveHelp;
use crate::completion_item::CompletionItem;
use crate::context::Context;
use crate::error::Result;
use std::borrow::Cow;

/// Memory-optimized completion result
///
/// This structure uses `CompletionItem` instead of parallel vectors,
/// reducing memory fragmentation and improving cache locality.
#[derive(Clone, Debug)]
pub struct CompletionResultOptimized {
    /// The completion items to suggest
    pub items: Vec<CompletionItem>,
    /// `ActiveHelp` messages to display
    pub active_help: Vec<ActiveHelp>,
}

impl CompletionResultOptimized {
    /// Creates a new empty completion result
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active_help: Vec::new(),
        }
    }

    /// Adds a completion value without a description
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add(mut self, value: impl Into<Cow<'static, str>>) -> Self {
        self.items.push(CompletionItem::new(value));
        self
    }

    /// Adds a completion value with a description
    #[must_use]
    pub fn add_with_description(
        mut self,
        value: impl Into<Cow<'static, str>>,
        desc: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.items
            .push(CompletionItem::with_description(value, desc));
        self
    }

    /// Adds multiple completion values without descriptions
    #[must_use]
    pub fn extend<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'static, str>>,
    {
        self.items
            .extend(values.into_iter().map(|v| CompletionItem::new(v)));
        self
    }

    /// Adds multiple completion items
    #[must_use]
    pub fn extend_items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = CompletionItem>,
    {
        self.items.extend(items);
        self
    }

    /// Adds an `ActiveHelp` message
    #[must_use]
    pub fn add_help(mut self, help: ActiveHelp) -> Self {
        self.active_help.push(help);
        self
    }

    /// Adds an `ActiveHelp` message from a string
    #[must_use]
    pub fn add_help_text<S: Into<String>>(mut self, message: S) -> Self {
        self.active_help.push(ActiveHelp::new(message));
        self
    }

    /// Adds a conditional `ActiveHelp` message
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

    /// Merges two completion results
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.items.extend(other.items);
        self.active_help.extend(other.active_help);
        self
    }

    /// Converts to the old `CompletionResult` format for compatibility
    #[must_use]
    pub fn into_legacy(self) -> crate::completion::CompletionResult {
        let mut result = crate::completion::CompletionResult::new();
        for item in self.items {
            result.values.push(item.value.into_owned());
            result
                .descriptions
                .push(item.description.map_or(String::new(), Cow::into_owned));
        }
        result.active_help = self.active_help;
        result
    }

    /// Creates from the old `CompletionResult` format
    #[must_use]
    pub fn from_legacy(legacy: crate::completion::CompletionResult) -> Self {
        let items = legacy
            .values
            .into_iter()
            .zip(legacy.descriptions)
            .map(|(value, desc)| {
                if desc.is_empty() {
                    CompletionItem::new(value)
                } else {
                    CompletionItem::with_description(value, desc)
                }
            })
            .collect();

        Self {
            items,
            active_help: legacy.active_help,
        }
    }
}

impl Default for CompletionResultOptimized {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for optimized completion functions
pub type CompletionFuncOptimized =
    Box<dyn Fn(&Context, &str) -> Result<CompletionResultOptimized> + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_completion_result() {
        let result = CompletionResultOptimized::new()
            .add("option1")
            .add_with_description("option2", "Description for option2")
            .extend(["option3", "option4"]);

        assert_eq!(result.items.len(), 4);
        assert_eq!(result.items[0].value, "option1");
        assert_eq!(result.items[0].description, None);
        assert_eq!(result.items[1].value, "option2");
        assert_eq!(
            result.items[1].description.as_deref(),
            Some("Description for option2")
        );
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that we can use static strings without allocation
        let result = CompletionResultOptimized::new()
            .add("static1")
            .add_with_description("static2", "static description");

        // These should be using Cow::Borrowed, not Cow::Owned
        assert!(matches!(result.items[0].value, Cow::Borrowed(_)));
        assert!(matches!(
            result.items[1].description.as_ref().unwrap(),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn test_legacy_conversion() {
        let optimized = CompletionResultOptimized::new()
            .add("val1")
            .add_with_description("val2", "desc2");

        let legacy = optimized.clone().into_legacy();
        assert_eq!(legacy.values, vec!["val1", "val2"]);
        assert_eq!(legacy.descriptions, vec!["", "desc2"]);

        let back = CompletionResultOptimized::from_legacy(legacy);
        assert_eq!(back.items.len(), optimized.items.len());
        assert_eq!(back.items[0].value, optimized.items[0].value);
    }
}
