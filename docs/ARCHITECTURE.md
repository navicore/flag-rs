# Architecture

Flag-rs is a zero-dependency Rust CLI framework. It lives in a single crate published as `flag-rs` on crates.io (v0.8.4, edition 2024).

## Module Map

All source lives under `src/`. There are no workspace members or internal crates.

### Core

| Module | Responsibility |
|--------|---------------|
| `command.rs` | `Command` struct and `CommandBuilder`. Owns subcommand tree, flag definitions, run function, lifecycle hooks, help rendering, and argument parsing/dispatch. This is the largest module (~1600 lines). |
| `context.rs` | `Context` passed to run functions. Holds parsed flag values and positional args. |
| `flag.rs` | `Flag`, `FlagType`, `FlagValue`, `FlagConstraint`. Defines flag metadata, value types (String, Bool, Int, Float, Choice, Range, File, Directory, StringSlice, StringArray), and inter-flag constraints (RequiredIf, ConflictsWith, Requires). |
| `error.rs` | `Error` enum and `Result` type alias. Variants: CommandNotFound, SubcommandRequired, FlagParsing, ArgumentParsing, Validation, Completion, Io, Custom. |
| `validator.rs` | `ArgValidator` trait for custom argument validation on commands. |

### Completion System

| Module | Responsibility |
|--------|---------------|
| `completion.rs` | `CompletionResult` and `CompletionFunc` type. Core completion logic invoked during shell tab-completion. |
| `shell.rs` | `Shell` enum (Bash, Zsh, Fish). Generates shell-specific completion scripts via `Command::generate_completion()`. |
| `completion_format.rs` | `CompletionFormat` for formatting completion output per shell (plain vs described). |
| `completion_cache.rs` | `CompletionCache` with TTL-based caching for expensive completion functions. |
| `completion_timeout.rs` | `make_timeout_completion()` wrapper that aborts slow completion functions. |
| `completion_item.rs` | `CompletionItem` - a value+description pair for rich completions. |
| `completion_optimized.rs` | `CompletionResultOptimized` using `Cow<str>` to avoid allocations. |
| `active_help.rs` | ActiveHelp system for contextual hints during completion. |

### UX & Output

| Module | Responsibility |
|--------|---------------|
| `color.rs` | ANSI color helpers. Respects `NO_COLOR` env var and TTY detection. |
| `terminal.rs` | Terminal width detection, text wrapping, help entry formatting. |
| `suggestion.rs` | Levenshtein-based "did you mean?" suggestions for typos. |

### Performance

| Module | Responsibility |
|--------|---------------|
| `string_pool.rs` | Global string interning pool to deduplicate repeated strings. |
| `parse_optimized.rs` | Memory-optimized flag parsing path. |

## Key Data Flow

1. **Build phase**: User constructs a `Command` tree via `CommandBuilder`, attaching flags, subcommands, run functions, and completion functions.
2. **Execute phase**: `Command::execute(args)` walks the subcommand tree matching args, parses flags into a `Context`, validates constraints, runs lifecycle hooks (pre-run/post-run), then calls the matched command's run function.
3. **Completion phase**: When the shell invokes the binary with completion env vars set, `Command` detects this, walks the tree to the current position, and calls the appropriate `CompletionFunc` to produce candidates. Output is formatted per `CompletionFormat` for the active shell.

## Test Structure

- `tests/` - Integration tests covering completion edge cases, lifecycle hooks, shell completion, validation, and general command execution.
- `src/completion_format_tests.rs` - Unit tests for completion formatting (included via `#[cfg(test)]`).

## Examples

- `examples/kubectl.rs` - Single-file kubectl-like CLI with three-level nesting and dynamic completions.
- `examples/kubectl_modular/` - Multi-file modular version demonstrating self-registering commands.
- Various `*_demo.rs` examples covering specific features (caching, timeouts, flags, hooks, etc.).
