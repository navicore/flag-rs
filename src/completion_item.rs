//! Memory-efficient completion item structure
//!
//! This module provides a more memory-efficient way to store completion items
//! by using a single struct instead of parallel vectors.

use std::borrow::Cow;

/// A single completion item with optional description
#[derive(Clone, Debug)]
pub struct CompletionItem {
    /// The completion value
    pub value: Cow<'static, str>,
    /// Optional description for the completion
    pub description: Option<Cow<'static, str>>,
}

impl CompletionItem {
    /// Creates a new completion item without description
    #[must_use]
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value: value.into(),
            description: None,
        }
    }

    /// Creates a new completion item with description
    #[must_use]
    pub fn with_description(
        value: impl Into<Cow<'static, str>>,
        desc: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            value: value.into(),
            description: Some(desc.into()),
        }
    }

    /// Returns the description or an empty string
    #[must_use]
    pub fn description_or_empty(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }
}

impl From<&'static str> for CompletionItem {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CompletionItem {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<(&'static str, &'static str)> for CompletionItem {
    fn from((value, desc): (&'static str, &'static str)) -> Self {
        Self::with_description(value, desc)
    }
}
