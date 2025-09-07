//! Argument validation for commands
//!
//! This module provides validation capabilities for command arguments,
//! allowing commands to enforce constraints on the number and type of
//! arguments they accept.

use crate::error::{Error, Result};

/// Type alias for custom validation functions
pub type ValidatorFn = dyn Fn(&[String]) -> Result<()> + Send + Sync;

/// Defines validation rules for command arguments
#[derive(Clone)]
pub enum ArgValidator {
    /// Exactly N arguments required
    ExactArgs(usize),
    /// At least N arguments required
    MinimumArgs(usize),
    /// At most N arguments allowed
    MaximumArgs(usize),
    /// Between min and max arguments (inclusive)
    RangeArgs(usize, usize),
    /// Arguments must be in the valid args list
    OnlyValidArgs(Vec<String>),
    /// Custom validation function
    Custom(std::sync::Arc<ValidatorFn>),
}

impl ArgValidator {
    /// Validates the given arguments against this validator
    pub fn validate(&self, args: &[String]) -> Result<()> {
        match self {
            Self::ExactArgs(expected) => {
                if args.len() != *expected {
                    return Err(Error::ArgumentValidation {
                        message: format!("accepts {} arg(s), received {}", expected, args.len()),
                        expected: expected.to_string(),
                        received: args.len(),
                    });
                }
                Ok(())
            }
            Self::MinimumArgs(min) => {
                if args.len() < *min {
                    return Err(Error::ArgumentValidation {
                        message: format!(
                            "requires at least {} arg(s), received {}",
                            min,
                            args.len()
                        ),
                        expected: format!("at least {min}"),
                        received: args.len(),
                    });
                }
                Ok(())
            }
            Self::MaximumArgs(max) => {
                if args.len() > *max {
                    return Err(Error::ArgumentValidation {
                        message: format!("accepts at most {} arg(s), received {}", max, args.len()),
                        expected: format!("at most {max}"),
                        received: args.len(),
                    });
                }
                Ok(())
            }
            Self::RangeArgs(min, max) => {
                if args.len() < *min || args.len() > *max {
                    return Err(Error::ArgumentValidation {
                        message: format!(
                            "accepts between {} and {} arg(s), received {}",
                            min,
                            max,
                            args.len()
                        ),
                        expected: format!("{min} to {max}"),
                        received: args.len(),
                    });
                }
                Ok(())
            }
            Self::OnlyValidArgs(valid_args) => {
                for arg in args {
                    if !valid_args.contains(arg) {
                        return Err(Error::ArgumentValidation {
                            message: format!("invalid argument \"{arg}\""),
                            expected: format!("one of: {}", valid_args.join(", ")),
                            received: 1,
                        });
                    }
                }
                Ok(())
            }
            Self::Custom(validator) => validator(args),
        }
    }
}

impl std::fmt::Debug for ArgValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExactArgs(n) => write!(f, "ExactArgs({n})"),
            Self::MinimumArgs(n) => write!(f, "MinimumArgs({n})"),
            Self::MaximumArgs(n) => write!(f, "MaximumArgs({n})"),
            Self::RangeArgs(min, max) => write!(f, "RangeArgs({min}, {max})"),
            Self::OnlyValidArgs(args) => write!(f, "OnlyValidArgs({args:?})"),
            Self::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_args() {
        let validator = ArgValidator::ExactArgs(2);

        // Should pass with exactly 2 args
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string()])
                .is_ok()
        );

        // Should fail with wrong number of args
        assert!(validator.validate(&["arg1".to_string()]).is_err());
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string(), "arg3".to_string()])
                .is_err()
        );
    }

    #[test]
    fn test_minimum_args() {
        let validator = ArgValidator::MinimumArgs(2);

        // Should pass with 2 or more args
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string()])
                .is_ok()
        );
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string(), "arg3".to_string()])
                .is_ok()
        );

        // Should fail with fewer args
        assert!(validator.validate(&["arg1".to_string()]).is_err());
        assert!(validator.validate(&[]).is_err());
    }

    #[test]
    fn test_maximum_args() {
        let validator = ArgValidator::MaximumArgs(2);

        // Should pass with 2 or fewer args
        assert!(validator.validate(&[]).is_ok());
        assert!(validator.validate(&["arg1".to_string()]).is_ok());
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string()])
                .is_ok()
        );

        // Should fail with more args
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string(), "arg3".to_string()])
                .is_err()
        );
    }

    #[test]
    fn test_range_args() {
        let validator = ArgValidator::RangeArgs(1, 3);

        // Should pass within range
        assert!(validator.validate(&["arg1".to_string()]).is_ok());
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string()])
                .is_ok()
        );
        assert!(
            validator
                .validate(&["arg1".to_string(), "arg2".to_string(), "arg3".to_string()])
                .is_ok()
        );

        // Should fail outside range
        assert!(validator.validate(&[]).is_err());
        assert!(
            validator
                .validate(&[
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string()
                ])
                .is_err()
        );
    }

    #[test]
    fn test_only_valid_args() {
        let validator = ArgValidator::OnlyValidArgs(vec![
            "start".to_string(),
            "stop".to_string(),
            "restart".to_string(),
        ]);

        // Should pass with valid args
        assert!(validator.validate(&["start".to_string()]).is_ok());
        assert!(
            validator
                .validate(&["stop".to_string(), "restart".to_string()])
                .is_ok()
        );

        // Should fail with invalid args
        assert!(validator.validate(&["invalid".to_string()]).is_err());
        assert!(
            validator
                .validate(&["start".to_string(), "invalid".to_string()])
                .is_err()
        );
    }

    #[test]
    fn test_custom_validator() {
        let validator = ArgValidator::Custom(std::sync::Arc::new(|args| {
            if args.iter().all(|arg| arg.parse::<i32>().is_ok()) {
                Ok(())
            } else {
                Err(Error::ArgumentValidation {
                    message: "all arguments must be integers".to_string(),
                    expected: "integers".to_string(),
                    received: args.len(),
                })
            }
        }));

        // Should pass with all integers
        assert!(
            validator
                .validate(&["1".to_string(), "2".to_string(), "3".to_string()])
                .is_ok()
        );

        // Should fail with non-integers
        assert!(
            validator
                .validate(&["1".to_string(), "abc".to_string()])
                .is_err()
        );
    }
}
