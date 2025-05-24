use crate::context::Context;
use crate::error::Result;

#[derive(Clone, Debug)]
pub struct CompletionResult {
    pub values: Vec<String>,
    pub descriptions: Vec<String>,
}

impl CompletionResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            descriptions: Vec::new(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add(mut self, value: impl Into<String>) -> Self {
        self.values.push(value.into());
        self.descriptions.push(String::new());
        self
    }

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
