//! Declarative macros for simplified CLI construction
//!
//! This module provides a set of macros that make it easy to define commands, flags,
//! and completions in a readable, declarative style while reducing boilerplate code.

/// Creates a static completion function with predefined values and descriptions
///
/// # Examples
///
/// ```rust
/// use flag_rs::completion;
///
/// // Simple completion with values and descriptions
/// completion! {
///     log_levels {
///         completions: [
///             ("debug", "Show all messages including debug"),
///             ("info", "Show informational messages and above"),
///             ("warn", "Show warnings and errors only"),
///             ("error", "Show errors only"),
///         ],
///         help: "Available log levels"
///     }
/// }
///
/// // Simple completion with just values
/// completion! {
///     environments {
///         completions: ["dev", "staging", "prod"]
///     }
/// }
/// ```
#[macro_export]
macro_rules! completion {
    // Completion with descriptions and help text
    ($name:ident {
        completions: [$(($value:expr, $desc:expr)),* $(,)?],
        help: $help_text:expr $(,)?
    }) => {
        fn $name() -> impl Fn(&$crate::Context, &str) -> $crate::Result<$crate::CompletionResult> + 'static {
            move |_ctx: &$crate::Context, prefix: &str| {
                let mut result = $crate::CompletionResult::new();

                // Add completions that match the prefix
                $(
                    if $value.starts_with(prefix) {
                        result = result.add_with_description($value.to_string(), $desc.to_string());
                    }
                )*

                // Add help text when no prefix is provided
                if prefix.is_empty() {
                    result = result.add_help_text($help_text);
                }

                Ok(result)
            }
        }
    };

    // Completion with just values (no descriptions)
    ($name:ident {
        completions: [$($value:expr),* $(,)?] $(,)?
    }) => {
        fn $name() -> impl Fn(&$crate::Context, &str) -> $crate::Result<$crate::CompletionResult> + 'static {
            move |_ctx: &$crate::Context, prefix: &str| {
                let mut result = $crate::CompletionResult::new();

                $(
                    if $value.starts_with(prefix) {
                        result = result.add($value.to_string());
                    }
                )*

                Ok(result)
            }
        }
    };
}

/// Creates a flag with sensible defaults and readable syntax
///
/// # Examples
///
/// ```rust
/// use flag_rs::flag;
///
/// // Boolean flag with short form
/// let verbose_flag = flag!(verbose(v): bool, default = false, usage = "Enable verbose output");
///
/// // String flag (completion requires a function, skipping for doctest)
/// let log_level_flag = flag!(log_level: string, default = "info", usage = "Set the logging level");
///
/// // Required integer flag
/// let port_flag = flag!(port(p): int, usage = "Port to listen on", required = true);
/// ```
#[macro_export]
macro_rules! flag {
    // Flag with short form and options
    ($name:ident($short:ident): bool, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::bool(stringify!($name))
                .short(stringify!($short).chars().next().unwrap());

            $(
                flag = flag!(@apply_option flag, $key, $value, bool);
            )+

            flag
        }
    };
    ($name:ident($short:ident): string, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::string(stringify!($name))
                .short(stringify!($short).chars().next().unwrap());

            $(
                flag = flag!(@apply_option flag, $key, $value, string);
            )+

            flag
        }
    };
    ($name:ident($short:ident): int, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::int(stringify!($name))
                .short(stringify!($short).chars().next().unwrap());

            $(
                flag = flag!(@apply_option flag, $key, $value, int);
            )+

            flag
        }
    };
    ($name:ident($short:ident): float, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::float(stringify!($name))
                .short(stringify!($short).chars().next().unwrap());

            $(
                flag = flag!(@apply_option flag, $key, $value, float);
            )+

            flag
        }
    };

    // Flag without short form
    ($name:ident: bool, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::bool(stringify!($name));

            $(
                flag = flag!(@apply_option flag, $key, $value, bool);
            )+

            flag
        }
    };
    ($name:ident: string, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::string(stringify!($name));

            $(
                flag = flag!(@apply_option flag, $key, $value, string);
            )+

            flag
        }
    };
    ($name:ident: int, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::int(stringify!($name));

            $(
                flag = flag!(@apply_option flag, $key, $value, int);
            )+

            flag
        }
    };
    ($name:ident: float, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut flag = $crate::Flag::float(stringify!($name));

            $(
                flag = flag!(@apply_option flag, $key, $value, float);
            )+

            flag
        }
    };

    // Helper to apply individual options
    (@apply_option $flag:expr, usage, $value:expr, $type:ident) => {
        $flag.usage($value)
    };
    (@apply_option $flag:expr, required, $value:expr, $type:ident) => {
        if $value { $flag.required() } else { $flag }
    };
    (@apply_option $flag:expr, completion, $value:expr, $type:ident) => {
        $flag.completion($value)
    };
    (@apply_option $flag:expr, default, $value:expr, bool) => {
        $flag.default_bool($value)
    };
    (@apply_option $flag:expr, default, $value:expr, string) => {
        $flag.default_str($value)
    };
    (@apply_option $flag:expr, default, $value:expr, int) => {
        $flag.default_int($value)
    };
    (@apply_option $flag:expr, default, $value:expr, float) => {
        $flag.default_float($value)
    };
}

/// Creates multiple flags at once for bulk flag definition
///
/// # Examples
///
/// ```rust
/// use flag_rs::{flags, flag};
///
/// let my_flags = flags![
///     verbose(v): bool, default = false, usage = "Enable verbose output";
///     port(p): int, default = 8080, usage = "Port to listen on";
///     config(c): string, usage = "Configuration file path", required = true;
/// ];
/// ```
#[macro_export]
macro_rules! flags {
    (
        $(
            $name:ident$(($short:ident))?: $type:ident, $($key:ident = $value:expr),+ $(,)?;
        )* $(;)?
    ) => {
        vec![
            $(
                flag!($name$(($short))?: $type, $($key = $value),+),
            )*
        ]
    };
}

#[cfg(test)]
mod tests {
    use crate::Context;

    #[test]
    fn test_completion_macro_with_descriptions() {
        completion! {
            test_completion {
                completions: [
                    ("debug", "Debug level"),
                    ("info", "Info level"),
                    ("warn", "Warn level"),
                ],
                help: "Available log levels"
            }
        }

        let completion_fn = test_completion();
        let ctx = Context::new(vec![]);

        // Test prefix matching
        let result = completion_fn(&ctx, "de").unwrap();
        assert!(result.values.iter().any(|v| v == "debug"));

        // Test help text when no prefix
        let result = completion_fn(&ctx, "").unwrap();
        assert!(!result.active_help.is_empty());
    }

    #[test]
    fn test_completion_macro_simple() {
        completion! {
            simple_completion {
                completions: ["dev", "staging", "prod"]
            }
        }

        let completion_fn = simple_completion();
        let ctx = Context::new(vec![]);

        let result = completion_fn(&ctx, "st").unwrap();
        assert!(result.values.iter().any(|v| v == "staging"));
    }

    #[test]
    fn test_flag_macro() {
        // Test boolean flag with default
        let bool_flag = flag!(verbose(v): bool, default = false, usage = "Enable verbose output");
        assert_eq!(bool_flag.name, "verbose");
        assert_eq!(bool_flag.short, Some('v'));

        // Test required string flag
        let string_flag = flag!(config: string, usage = "Config file", required = true);
        assert_eq!(string_flag.name, "config");
        assert!(string_flag.required);
    }

    #[test]
    fn test_flags_macro() {
        let flag_list = flags![
            verbose(v): bool, default = false, usage = "Enable verbose output";
            port(p): int, default = 8080, usage = "Port to listen on";
        ];

        assert_eq!(flag_list.len(), 2);
        assert_eq!(flag_list[0].name, "verbose");
        assert_eq!(flag_list[1].name, "port");
    }
}
