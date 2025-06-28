//! ActiveHelp system for contextual hints during completion
//!
//! ActiveHelp provides contextual assistance to users as they type commands,
//! similar to Cobra's ActiveHelp feature. Help messages can be static or
//! conditional based on the current command context.

use crate::context::Context;

/// Type alias for `ActiveHelp` condition functions
pub type ConditionFn = dyn Fn(&Context) -> bool + Send + Sync;

/// Represents an `ActiveHelp` message with optional condition
pub struct ActiveHelp {
    /// The help message to display
    pub message: String,
    /// Optional condition that must be true for this help to be shown
    pub condition: Option<std::sync::Arc<ConditionFn>>,
}

impl ActiveHelp {
    /// Creates a new `ActiveHelp` message without condition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::active_help::ActiveHelp;
    ///
    /// let help = ActiveHelp::new("Use TAB to see available options");
    /// ```
    #[must_use]
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            condition: None,
        }
    }

    /// Creates a new `ActiveHelp` message with a condition
    ///
    /// The help will only be shown when the condition returns true.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::active_help::ActiveHelp;
    ///
    /// let help = ActiveHelp::with_condition(
    ///     "Tip: Use -n <namespace> to filter results",
    ///     |ctx| ctx.flag("namespace").is_none()
    /// );
    /// ```
    #[must_use]
    pub fn with_condition<S, F>(message: S, condition: F) -> Self
    where
        S: Into<String>,
        F: Fn(&Context) -> bool + Send + Sync + 'static,
    {
        Self {
            message: message.into(),
            condition: Some(std::sync::Arc::new(condition)),
        }
    }

    /// Checks if this help should be displayed given the current context
    #[must_use]
    pub fn should_display(&self, ctx: &Context) -> bool {
        self.condition
            .as_ref()
            .map_or(true, |condition| condition(ctx))
    }
}

impl Clone for ActiveHelp {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            condition: self.condition.clone(),
        }
    }
}

impl std::fmt::Debug for ActiveHelp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveHelp")
            .field("message", &self.message)
            .field("condition", &self.condition.is_some())
            .finish()
    }
}

/// Configuration for `ActiveHelp` behavior
#[derive(Clone, Debug)]
pub struct ActiveHelpConfig {
    /// Whether to show help on double-TAB
    pub show_on_double_tab: bool,
    /// Whether to show help when no completions are available
    pub show_on_no_completions: bool,
    /// Whether to disable `ActiveHelp` globally
    pub disabled: bool,
    /// Environment variable to check for disabling `ActiveHelp`
    pub disable_env_var: Option<String>,
}

impl Default for ActiveHelpConfig {
    fn default() -> Self {
        Self {
            show_on_double_tab: true,
            show_on_no_completions: true,
            disabled: false,
            disable_env_var: Some("COBRA_ACTIVE_HELP".to_string()), // Compatible with Cobra
        }
    }
}

impl ActiveHelpConfig {
    /// Checks if `ActiveHelp` is enabled based on configuration and environment
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        if self.disabled {
            return false;
        }

        // Check environment variable if configured
        if let Some(ref env_var) = self.disable_env_var {
            if let Ok(value) = std::env::var(env_var) {
                // If set to "0" or "false", ActiveHelp is disabled
                return !matches!(value.to_lowercase().as_str(), "0" | "false");
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_help_new() {
        let help = ActiveHelp::new("Test message");
        assert_eq!(help.message, "Test message");
        assert!(help.condition.is_none());
    }

    #[test]
    fn test_active_help_with_condition() {
        let help = ActiveHelp::with_condition("Conditional help", |_ctx| true);
        assert_eq!(help.message, "Conditional help");
        assert!(help.condition.is_some());
    }

    #[test]
    fn test_should_display() {
        let ctx = Context::new(vec![]);

        // Help without condition should always display
        let help = ActiveHelp::new("Always shown");
        assert!(help.should_display(&ctx));

        // Help with always-true condition
        let help = ActiveHelp::with_condition("Also shown", |_| true);
        assert!(help.should_display(&ctx));

        // Help with always-false condition
        let help = ActiveHelp::with_condition("Never shown", |_| false);
        assert!(!help.should_display(&ctx));
    }

    #[test]
    fn test_active_help_config_default() {
        let config = ActiveHelpConfig::default();
        assert!(config.show_on_double_tab);
        assert!(config.show_on_no_completions);
        assert!(!config.disabled);
        assert_eq!(
            config.disable_env_var,
            Some("COBRA_ACTIVE_HELP".to_string())
        );
    }

    #[test]
    fn test_active_help_config_is_enabled() {
        // Default config should be enabled
        let config = ActiveHelpConfig::default();
        assert!(config.is_enabled());

        // Disabled config
        let config = ActiveHelpConfig {
            disabled: true,
            ..Default::default()
        };
        assert!(!config.is_enabled());
    }
}
