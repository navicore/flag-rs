//! Declarative macros for simplified CLI construction
//!
//! This module provides a set of macros that make it easy to define commands, flags,
//! and completions in a readable, declarative style while reducing boilerplate code.

/// Creates completion functions with support for both static and dynamic completions
///
/// # Examples
///
/// ```rust
/// use flag_rs::completion;
///
/// // Static completion with values and descriptions
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
/// // Simple static completion with just values
/// completion! {
///     environments {
///         completions: ["dev", "staging", "prod"]
///     }
/// }
///
/// // Dynamic completion with closure
/// completion! {
///     active_sessions {
///         dynamic: |ctx, prefix| {
///             // Your custom completion logic here
///             let sessions = vec!["session1", "session2", "session3"];
///             let mut result = flag_rs::CompletionResult::new();
///             for session in sessions {
///                 if session.starts_with(prefix) {
///                     result = result.add_with_description(
///                         session.to_string(),
///                         "Active session".to_string()
///                     );
///                 }
///             }
///             if prefix.is_empty() {
///                 result = result.add_help_text("Currently active sessions");
///             }
///             Ok(result)
///         }
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

    // Dynamic completion with closure
    ($name:ident {
        dynamic: $closure:expr $(,)?
    }) => {
        fn $name() -> impl Fn(&$crate::Context, &str) -> $crate::Result<$crate::CompletionResult> + 'static {
            move |ctx: &$crate::Context, prefix: &str| {
                let completion_fn: fn(&$crate::Context, &str) -> $crate::Result<$crate::CompletionResult> = $closure;
                completion_fn(ctx, prefix)
            }
        }
    };

    // Dynamic completion with closure and help text
    ($name:ident {
        dynamic: $closure:expr,
        help: $help_text:expr $(,)?
    }) => {
        fn $name() -> impl Fn(&$crate::Context, &str) -> $crate::Result<$crate::CompletionResult> + 'static {
            move |ctx: &$crate::Context, prefix: &str| {
                let completion_fn: fn(&$crate::Context, &str) -> $crate::Result<$crate::CompletionResult> = $closure;
                let mut result = completion_fn(ctx, prefix)?;

                // Add help text when no prefix is provided
                if prefix.is_empty() {
                    result = result.add_help_text($help_text);
                }

                Ok(result)
            }
        }
    };
}

/// Creates a complete command with flags, subcommands, and run handler in one declaration
///
/// # Examples
///
/// ```rust
/// use flag_rs::command;
///
/// // Simple command with flags and run handler
/// command! {
///     myapp {
///         short: "A demo CLI application",
///         long: "This is my awesome CLI app with full macro support",
///         
///         flags: [
///             verbose(v): bool = false, usage = "Enable verbose output";
///             config(c): string, usage = "Config file", required = true;
///         ],
///         
///         run: |ctx| {
///             let verbose = ctx.flag_bool("verbose").unwrap_or(false);
///             let config = ctx.flag("config").expect("Config required");
///             println!("Starting with config: {}", config);
///             Ok(())
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! command {
    // Full command with all features
    ($name:ident {
        short: $short_desc:expr,
        long: $long_desc:expr,
        flags: [$($flag_name:ident$(($flag_short:ident))?: $flag_type:ident $(= $flag_default:expr)?, usage = $flag_usage:expr $(, completion = $flag_completion:expr)? $(, required = $flag_required:expr)?);* $(;)?],
        $(subcommands: [$($subcommand:ident),* $(,)?],)?
        run: $run_handler:expr $(,)?
    }) => {
        fn $name() -> $crate::Command {
            let mut builder = $crate::CommandBuilder::new(stringify!($name))
                .short($short_desc)
                .long($long_desc);

            // Add flags
            $(
                builder = builder.flag(command!(@build_flag $flag_name$(($flag_short))?, $flag_type $(= $flag_default)?, usage = $flag_usage $(, completion = $flag_completion)? $(, required = $flag_required)?));
            )*

            // Add subcommands if specified
            $($(
                builder = builder.subcommand($subcommand());
            )*)?

            builder.run($run_handler).build()
        }
    };

    // Command without subcommands
    ($name:ident {
        short: $short_desc:expr,
        long: $long_desc:expr,
        flags: [$($flag_name:ident$(($flag_short:ident))?: $flag_type:ident $(= $flag_default:expr)?, usage = $flag_usage:expr $(, completion = $flag_completion:expr)? $(, required = $flag_required:expr)?);* $(;)?],
        run: $run_handler:expr $(,)?
    }) => {
        fn $name() -> $crate::Command {
            let mut builder = $crate::CommandBuilder::new(stringify!($name))
                .short($short_desc)
                .long($long_desc);

            // Add flags
            $(
                builder = builder.flag(command!(@build_flag $flag_name$(($flag_short))?, $flag_type $(= $flag_default)?, usage = $flag_usage $(, completion = $flag_completion)? $(, required = $flag_required)?));
            )*

            builder.run($run_handler).build()
        }
    };

    // Simple command with just run handler (no flags)
    ($name:ident {
        short: $short_desc:expr,
        long: $long_desc:expr,
        run: $run_handler:expr $(,)?
    }) => {
        fn $name() -> $crate::Command {
            $crate::CommandBuilder::new(stringify!($name))
                .short($short_desc)
                .long($long_desc)
                .run($run_handler)
                .build()
        }
    };

    // Helper macros to build flags within command context
    (@build_flag $name:ident($short:ident), bool = $default:expr, usage = $usage:expr) => {
        flag!($name($short): bool, default = $default, usage = $usage)
    };
    (@build_flag $name:ident($short:ident), bool = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name($short): bool, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident($short:ident), bool = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name($short): bool, default = $default, usage = $usage, required = $required)
    };
    (@build_flag $name:ident($short:ident), string = $default:expr, usage = $usage:expr) => {
        flag!($name($short): string, default = $default, usage = $usage)
    };
    (@build_flag $name:ident($short:ident), string = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name($short): string, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident($short:ident), string = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name($short): string, default = $default, usage = $usage, required = $required)
    };
    (@build_flag $name:ident($short:ident), int = $default:expr, usage = $usage:expr) => {
        flag!($name($short): int, default = $default, usage = $usage)
    };
    (@build_flag $name:ident($short:ident), int = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name($short): int, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident($short:ident), int = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name($short): int, default = $default, usage = $usage, required = $required)
    };
    (@build_flag $name:ident($short:ident), float = $default:expr, usage = $usage:expr) => {
        flag!($name($short): float, default = $default, usage = $usage)
    };
    (@build_flag $name:ident($short:ident), float = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name($short): float, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident($short:ident), float = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name($short): float, default = $default, usage = $usage, required = $required)
    };

    // Without short form
    (@build_flag $name:ident, bool = $default:expr, usage = $usage:expr) => {
        flag!($name: bool, default = $default, usage = $usage)
    };
    (@build_flag $name:ident, bool = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name: bool, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident, bool = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name: bool, default = $default, usage = $usage, required = $required)
    };
    (@build_flag $name:ident, string = $default:expr, usage = $usage:expr) => {
        flag!($name: string, default = $default, usage = $usage)
    };
    (@build_flag $name:ident, string = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name: string, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident, string = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name: string, default = $default, usage = $usage, required = $required)
    };
    (@build_flag $name:ident, int = $default:expr, usage = $usage:expr) => {
        flag!($name: int, default = $default, usage = $usage)
    };
    (@build_flag $name:ident, int = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name: int, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident, int = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name: int, default = $default, usage = $usage, required = $required)
    };
    (@build_flag $name:ident, float = $default:expr, usage = $usage:expr) => {
        flag!($name: float, default = $default, usage = $usage)
    };
    (@build_flag $name:ident, float = $default:expr, usage = $usage:expr, completion = $completion:expr) => {
        flag!($name: float, default = $default, usage = $usage, completion = $completion)
    };
    (@build_flag $name:ident, float = $default:expr, usage = $usage:expr, required = $required:expr) => {
        flag!($name: float, default = $default, usage = $usage, required = $required)
    };

    // Required flags without defaults
    (@build_flag $name:ident($short:ident), $flag_type:ident, usage = $usage:expr, required = true) => {
        flag!($name($short): $flag_type, usage = $usage, required = true)
    };
    (@build_flag $name:ident, $flag_type:ident, usage = $usage:expr, required = true) => {
        flag!($name: $flag_type, usage = $usage, required = true)
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

    #[test]
    fn test_dynamic_completion_macro() {
        // Test dynamic completion without help
        completion! {
            dynamic_test {
                dynamic: |_ctx, prefix| {
                    let mut result = crate::CompletionResult::new();
                    let items = vec!["apple", "apricot", "banana"];

                    for item in items {
                        if item.starts_with(prefix) {
                            result = result.add_with_description(
                                item.to_string(),
                                format!("{item} fruit")
                            );
                        }
                    }

                    Ok(result)
                }
            }
        }

        let completion_fn = dynamic_test();
        let ctx = Context::new(vec![]);

        // Test prefix matching
        let result = completion_fn(&ctx, "ap").unwrap();
        assert_eq!(result.values.len(), 2);
        assert!(result.values.iter().any(|v| v == "apple"));
        assert!(result.values.iter().any(|v| v == "apricot"));
        assert!(!result.values.iter().any(|v| v == "banana"));

        // Test no prefix returns all
        let result = completion_fn(&ctx, "").unwrap();
        assert_eq!(result.values.len(), 3);
    }

    #[test]
    fn test_dynamic_completion_with_help() {
        // Test dynamic completion with help text
        completion! {
            dynamic_with_help {
                dynamic: |_ctx, prefix| {
                    let mut result = crate::CompletionResult::new();
                    let sessions = vec!["session1", "session2", "session3"];

                    for session in sessions {
                        if session.starts_with(prefix) {
                            result = result.add_with_description(
                                session.to_string(),
                                "Active session".to_string()
                            );
                        }
                    }

                    Ok(result)
                },
                help: "Currently active sessions"
            }
        }

        let completion_fn = dynamic_with_help();
        let ctx = Context::new(vec![]);

        // Test that help is added when no prefix
        let result = completion_fn(&ctx, "").unwrap();
        assert!(!result.active_help.is_empty());
        assert_eq!(result.values.len(), 3);

        // Test prefix filtering still works
        let result = completion_fn(&ctx, "session1").unwrap();
        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values[0], "session1");
    }

    #[test]
    fn test_simple_command_macro() {
        // Test simple command without flags
        command! {
            simple_cmd {
                short: "A simple test command",
                long: "This is a simple test command for the macro",
                run: |_ctx| {
                    Ok(())
                }
            }
        }

        let cmd = simple_cmd();
        assert_eq!(cmd.name(), "simple_cmd");
        assert_eq!(cmd.short(), "A simple test command");
    }

    #[test]
    fn test_command_macro_with_flags() {
        // Mock completion function for testing
        completion! {
            test_levels {
                completions: ["low", "medium", "high"]
            }
        }

        // Test command with flags
        command! {
            complex_cmd {
                short: "A complex test command",
                long: "This command has flags and demonstrates the macro",
                flags: [
                    verbose(v): bool = false, usage = "Enable verbose output";
                    level(l): string = "medium", usage = "Set the level", completion = test_levels();
                    count: int = 10, usage = "Number of items";
                ],
                run: |ctx| {
                    let _verbose = ctx.flag_bool("verbose").unwrap_or(false);
                    let _level = ctx.flag_str_or("level", "medium");
                    let _count = ctx.flag_int("count").unwrap_or(10);
                    Ok(())
                }
            }
        }

        let cmd = complex_cmd();
        assert_eq!(cmd.name(), "complex_cmd");
        assert_eq!(cmd.short(), "A complex test command");
        assert_eq!(cmd.flags().len(), 3);

        // Check flag names
        let flag_names: Vec<&String> = cmd.flags().keys().collect();
        assert!(flag_names.contains(&&"verbose".to_string()));
        assert!(flag_names.contains(&&"level".to_string()));
        assert!(flag_names.contains(&&"count".to_string()));
    }
}
