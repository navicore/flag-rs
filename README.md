# Flag-rs - A Cobra-inspired CLI Framework for Rust

Flag-rs is a command-line interface (CLI) framework for Rust inspired by Go's Cobra
library. It provides dynamic command completion, self-registering commands, and
a clean, modular architecture for building sophisticated CLI applications.

## Key Features

- **Zero Dependencies** - Pure Rust implementation with no external crates
- **Dynamic Runtime Completions** - Generate completions based on runtime state (like kubectl)
- **Self-Registering Commands** - Commands can register themselves with parent commands
- **Modular Architecture** - Organize commands in separate files/modules
- **Shell Completion Support** - Generate completion scripts for bash, zsh, and fish
- **Hierarchical Flag Inheritance** - Global flags are available to all subcommands
- **Idiomatic Error Handling** - Uses standard Rust `Result` types
- **Colored Output** - Beautiful help messages with ANSI color support (respects NO_COLOR and terminal detection)

## Why Flag-rs?

Flag-rs enables dynamic completions that can query APIs, databases, or any runtime
state - just like `kubectl` does when completing pod names.  I've struggled and
failed for a long time at modifying the leading command line processing crate to
work this way - hence this new crate.

## Why Not Flag-rs?

The Flag-rs implementation may be naive - the leading crate,
[Clap](https://github.com/clap-rs/clap), is very well regarded and meets the
needs of a huge knowledgeable user base.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
flag-rs = "0.6"
```

## Quick Start

```rust
use flag_rs::{Command, CommandBuilder, Context, Flag, FlagType, FlagValue, CompletionResult};

fn main() {
    let app = CommandBuilder::new("myapp")
        .short("A simple CLI application")
        .long("This is a longer description of my application")
        .flag(
            Flag::new("verbose")
                .short('v')
                .usage("Enable verbose output")
                .value_type(FlagType::Bool)
                .default(FlagValue::Bool(false))
        )
        .subcommand(build_serve_command())
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn build_serve_command() -> Command {
    CommandBuilder::new("serve")
        .short("Start the server")
        .flag(
            Flag::new("port")
                .short('p')
                .usage("Port to listen on")
                .value_type(FlagType::Int)
                .default(FlagValue::Int(8080))
        )
        .run(|ctx| {
            let port = ctx.flag("port")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(8080);

            println!("Starting server on port {}", port);
            Ok(())
        })
        .build()
}
```

## Dynamic Completions

The killer feature of Flag is dynamic completions. Unlike static completions,
these run at completion time and can return different values based on current
state:

```rust
CommandBuilder::new("get")
    .arg_completion(|ctx, prefix| {
        // This runs when the user presses TAB!
        let namespace = ctx.flag("namespace").unwrap_or("default");

        // In a real app, you'd query an API here
        let items = fetch_items_from_api(namespace);

        Ok(CompletionResult::new()
            .extend(items.into_iter()
                .filter(|item| item.starts_with(prefix))
                .collect::<Vec<_>>()))
    })
    .build()
```

## Modular Command Structure

For larger applications, Flag supports a modular structure where commands
register themselves:

```rust
// src/cmd/mod.rs
pub fn register_commands(root: &mut Command) {
    serve::register(root);
    deploy::register(root);
    status::register(root);
}

// src/cmd/serve.rs
pub fn register(parent: &mut Command) {
    let cmd = CommandBuilder::new("serve")
        .short("Start the server")
        .run(|ctx| {
            // Implementation
            Ok(())
        })
        .build();

    parent.add_command(cmd);
}
```

## Shell Completions

Generate completion scripts for popular shells:

```rust
// Add a completion command to your CLI
CommandBuilder::new("completion")
    .short("Generate shell completion scripts")
    .run(|ctx| {
        let shell = ctx.args().first()
            .ok_or("Shell name required")?;

        let script = match shell.as_str() {
            "bash" => app.generate_completion(Shell::Bash),
            "zsh" => app.generate_completion(Shell::Zsh),
            "fish" => app.generate_completion(Shell::Fish),
            _ => return Err("Unsupported shell"),
        };

        println!("{}", script);
        Ok(())
    })
    .build()
```

Then users can enable completions:

```bash
# Bash
source <(myapp completion bash)

# Zsh
source <(myapp completion zsh)

# Fish
myapp completion fish | source
```

## Flag Types

Flag-rs supports multiple value types:

- `String` - Text values
- `Bool` - Boolean flags
- `Int` - Integer values
- `Float` - Floating point values
- `StringSlice` - Multiple string values

## Error Handling

Flag uses idiomatic Rust error handling with no forced dependencies:

```rust
pub enum Error {
    CommandNotFound(String),
    SubcommandRequired(String),
    FlagParsing(String),
    ArgumentParsing(String),
    ValidationError(String),
    Completion(String),
    Io(std::io::Error),
    Custom(Box<dyn std::error::Error + Send + Sync>),
}
```

## Examples

See the `examples/` directory for complete examples:

- `kubectl.rs` - A simple kubectl-like CLI demonstrating dynamic completions
- `kubectl_modular/` - A modular kubectl implementation showing command self-registration

## Design Philosophy

Flag is designed to be a foundational library with zero dependencies. This ensures:

- Fast compilation times
- No dependency conflicts
- Maximum flexibility for users
- Minimal binary size

Users can add their own dependencies for features like colored output, async
runtime, or configuration file support without conflicts.

## Development

### Running Clippy Locally

Flag uses opinionated clippy linting to maintain code quality. Here are the most useful commands:

```bash
# Basic clippy check
cargo clippy

# Strict clippy with all lints (what CI uses)
cargo clippy -- \
  -D clippy::all \
  -D clippy::pedantic \
  -D clippy::nursery \
  -D clippy::cargo

# Fix clippy warnings automatically
cargo clippy --fix

# Check specific target
cargo clippy --tests     # Just tests
cargo clippy --examples  # Just examples
cargo clippy --all       # Everything
```

### Continuous Integration

The project uses GitHub Actions for CI with:
- Rust stable toolchain
- Format checking (`cargo fmt`)
- Comprehensive clippy linting
- Test execution
- Documentation build verification

The clippy configuration in CI is very strict to catch potential issues early.
See `.github/workflows/ci.yml` for the full configuration.

### Publishing Releases

To publish a new release to crates.io:

1. Update the version in `Cargo.toml`
2. Commit and push the version change
3. Create a GitHub release with a tag like `v0.6.0` (matching the Cargo.toml version)
4. GitHub Actions will automatically publish to crates.io

**Setup Required:**
- Add your crates.io API token as a GitHub secret named `CRATES_IO_TOKEN`
- Get your token from: https://crates.io/settings/tokens

## License

MIT
