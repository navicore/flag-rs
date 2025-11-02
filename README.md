# Flag-rs - A Cobra-inspired CLI Framework for Rust

Flag-rs is a command-line interface (CLI) framework for Rust inspired by Go's Cobra
library. It provides dynamic command completion, self-registering commands, and
a clean, modular architecture for building sophisticated CLI applications.

## Key Features

- **Zero Dependencies** - Pure Rust implementation with no external crates
- **Multi-Level Nested Commands** - Full support for commands like `myapp deploy status` with unlimited nesting
- **Dynamic Runtime Completions** - Generate completions based on runtime state (like kubectl)
- **Working Zsh Subcommand Completion** - Tab completion that actually works for nested commands in zsh, bash, and fish
- **Self-Registering Commands** - Commands can register themselves with parent commands
- **Modular Architecture** - Organize commands in separate files/modules
- **Hierarchical Flag Inheritance** - Global flags are available to all subcommands
- **Idiomatic Error Handling** - Uses standard Rust `Result` types
- **Colored Output** - Beautiful help messages with ANSI color support (respects NO_COLOR and terminal detection)

## Why Flag-rs?

Flag-rs solves specific pain points that other CLI frameworks struggle with:

1. **Dynamic completions that actually work** - Query APIs, databases, or any runtime state at tab-press time, just like `kubectl` does when completing pod names
2. **Reliable zsh subcommand completion** - Multi-level commands like `myapp deploy status` get proper tab completion in various shells.

## Why Not Flag-rs?

The Flag-rs implementation may be naive - the leading crate,
[Clap](https://github.com/clap-rs/clap), is very well regarded and meets the
needs of a huge knowledgeable user base. Choose clap if you need maximum stability,

extensive documentation, and don't require dynamic completions.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
flag-rs = "0.8"
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

## Nested Commands with Tab Completion

Flag-rs fully supports multi-level command hierarchies with working tab completion at every level. This is especially valuable for zsh users who have experienced issues with other frameworks.

### Building Nested Commands

Create commands like `myapp deploy status` and `myapp deploy publish`:

```rust
use flag_rs::{CommandBuilder, Context, CompletionResult};

fn main() {
    let app = CommandBuilder::new("myapp")
        .short("My application")
        .subcommand(build_deploy_command())
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    app.execute(args).unwrap();
}

fn build_deploy_command() -> Command {
    CommandBuilder::new("deploy")
        .short("Deployment commands")
        .subcommand(
            CommandBuilder::new("status")
                .short("Check deployment status")
                .arg_completion(|ctx, prefix| {
                    // Tab completion for deployment names
                    let deployments = vec!["web-api", "worker", "database"];
                    Ok(CompletionResult::new()
                        .extend(deployments.into_iter()
                            .filter(|d| d.starts_with(prefix))
                            .map(String::from)
                            .collect()))
                })
                .run(|ctx| {
                    let deployment = ctx.args().first()
                        .ok_or("Deployment name required")?;
                    println!("Status of {}: Running", deployment);
                    Ok(())
                })
                .build()
        )
        .subcommand(
            CommandBuilder::new("publish")
                .short("Publish a deployment")
                .run(|ctx| {
                    println!("Publishing deployment...");
                    Ok(())
                })
                .build()
        )
        .build()
}
```

### Tab Completion Behavior

With the above setup:

- `myapp <TAB>` → shows `deploy` (and any other root commands)
- `myapp deploy <TAB>` → shows `status`, `publish`
- `myapp deploy status <TAB>` → shows `web-api`, `worker`, `database`

This works correctly in **bash**, **zsh**, and **fish** shells.

### Arbitrary Nesting Depth

The architecture supports unlimited nesting levels:

```rust
CommandBuilder::new("myapp")           // Level 1
    .subcommand(
        CommandBuilder::new("cloud")    // Level 2
            .subcommand(
                CommandBuilder::new("compute")  // Level 3
                    .subcommand(
                        CommandBuilder::new("instances")  // Level 4
                            .subcommand(
                                CommandBuilder::new("list")  // Level 5
                                    .run(|ctx| { /* ... */ })
                                    .build()
                            )
                            .build()
                    )
                    .build()
            )
            .build()
    )
    .build()
```

Results in: `myapp cloud compute instances list` with tab completion at each level.

## Dynamic Completions

The killer feature of Flag-rs is dynamic completions. Unlike static completions,
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

### Completions with Descriptions

Flag-rs supports rich completions with descriptions, similar to Cobra. When
descriptions are provided, they are handled appropriately by each shell:

```rust
.flag_completion("environment", |_ctx, prefix| {
    Ok(CompletionResult::new()
        .add_with_description("dev", "Development environment - safe for testing")
        .add_with_description("staging", "Staging environment - production mirror")
        .add_with_description("prod", "Production environment - BE CAREFUL!"))
})
```

**Shell-specific behavior:**
- **Bash**: Shows only the completion values (descriptions not supported natively)
- **Zsh**: Shows descriptions using native format: `value:description`
- **Fish**: Shows descriptions using tab-separated format: `value[TAB]description`

For Zsh and Fish, descriptions appear alongside completions in the shell's native
format, providing helpful context when selecting options.

### Completion Caching

For expensive completion operations (API calls, file system scans), use the built-in
caching mechanism:

```rust
use flag_rs::completion_cache::CompletionCache;
use std::time::Duration;
use std::sync::Arc;

let cache = Arc::new(CompletionCache::new(Duration::from_secs(5)));

CommandBuilder::new("get")
    .arg_completion({
        let cache = Arc::clone(&cache);
        move |ctx, prefix| {
            let key = CompletionCache::make_key(
                &["get".to_string()],
                prefix,
                ctx.flags()
            );

            // Check cache first
            if let Some(cached) = cache.get(&key) {
                return Ok(cached);
            }

            // Expensive operation
            let result = fetch_from_api()?;

            // Cache the result
            cache.put(key, result.clone());
            Ok(result)
        }
    })
    .build()
```

### Completion Timeouts

Protect against slow completion functions that could hang the shell:

```rust
use flag_rs::completion_timeout::make_timeout_completion;
use std::time::Duration;

CommandBuilder::new("search")
    .arg_completion(make_timeout_completion(
        Duration::from_millis(100),
        |ctx, prefix| {
            // This will timeout if it takes > 100ms
            search_database(prefix)
        }
    ))
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

Flag-rs generates working completion scripts for bash, zsh, and fish. Unlike some frameworks, **nested subcommands work correctly in zsh**.

### Adding Completion Support

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

### Enabling Completions

Users can enable completions in their shell:

```bash
# Bash
source <(myapp completion bash)

# Zsh - works correctly for nested subcommands!
source <(myapp completion zsh)

# Fish
myapp completion fish | source
```

### What Works

All shells support:
- ✅ Root command completion
- ✅ Nested subcommand completion (including zsh!)
- ✅ Flag completion with descriptions (zsh and fish show descriptions)
- ✅ Dynamic argument completion
- ✅ Flag value completion

The completion system navigates the full command hierarchy, so typing `myapp deploy <TAB>` correctly shows subcommands at that level, even in zsh.

## Flag Types

Flag-rs supports multiple value types:

- `String` - Text values
- `Bool` - Boolean flags
- `Int` - Integer values
- `Float` - Floating point values
- `StringSlice` - Multiple string values
- `StringArray` - Array of string values
- `Choice(Vec<String>)` - Enumerated choices with validation
- `Range(i64, i64)` - Integer range validation
- `File` - File path validation
- `Directory` - Directory path validation

### Advanced Flag Features

#### Flag Constraints

Flag-rs supports advanced flag relationships:

```rust
use flag_rs::{Flag, FlagConstraint};

// Required if another flag is set
Flag::new("cert")
    .value_type(FlagType::File)
    .constraint(FlagConstraint::RequiredIf("tls".to_string()))

// Conflicts with other flags
Flag::new("json")
    .value_type(FlagType::Bool)
    .constraint(FlagConstraint::ConflictsWith(vec!["yaml".to_string(), "xml".to_string()]))

// Requires other flags
Flag::new("ssl-verify")
    .value_type(FlagType::Bool)
    .constraint(FlagConstraint::Requires(vec!["ssl".to_string()]))
```

#### Choice Validation

```rust
Flag::new("environment")
    .value_type(FlagType::Choice(vec![
        "dev".to_string(),
        "staging".to_string(),
        "prod".to_string()
    ]))
```

#### Range Validation

```rust
Flag::new("workers")
    .value_type(FlagType::Range(1, 100))
    .default(FlagValue::Int(4))
```

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

- **`kubectl.rs`** - A kubectl-like CLI demonstrating **three-level nested commands** (`kubectl get pods`) with dynamic completions and working zsh tab completion
- **`kubectl_modular/`** - A modular kubectl implementation showing command self-registration across multiple files
- `advanced_flags_demo.rs` - Demonstrates advanced flag types and constraints
- `caching_demo.rs` - Shows completion caching for expensive operations
- `timeout_demo.rs` - Demonstrates timeout protection for slow completions
- `memory_optimization_demo.rs` - Shows memory optimization techniques
- `benchmark.rs` - Performance benchmarking suite

**Quick test of nested commands with completion:**

```bash
# Build the kubectl example
cargo build --example kubectl

# Try the three-level command structure
./target/debug/examples/kubectl get pods nginx-7fb96c846b-8xvnl

# Generate and test zsh completion
source <(./target/debug/examples/kubectl completion zsh)
kubectl get <TAB>           # Shows: pods, services, deployments
kubectl get pods <TAB>      # Shows actual pod names with dynamic completion
```

## Performance & Memory Optimization

Flag-rs includes several performance optimizations for large-scale CLI applications:

### String Interning

Reduce memory usage for repeated strings:

```rust
use flag_rs::string_pool;

// Intern frequently used strings
let cmd_name = string_pool::intern("kubectl");
let flag_name = string_pool::intern("namespace");
```

### Optimized Completions

Use memory-efficient completion structures:

```rust
use flag_rs::completion_optimized::{CompletionResultOptimized, CompletionItem};
use std::borrow::Cow;

CompletionResultOptimized::new()
    .add_with_description(
        Cow::Borrowed("static-value"),  // No allocation
        Cow::Borrowed("Static description")
    )
```

### Performance Characteristics

Based on our benchmarks:

| Operation | Performance | Notes |
|-----------|-------------|-------|
| Simple command creation | ~500 ns | 2 flags |
| Complex CLI (50 subcommands) | ~150 μs | 500 total commands |
| Flag parsing | ~2 μs | Per flag |
| Subcommand lookup | ~50 ns | O(1) HashMap |
| Dynamic completion | ~15 μs | 100 items |
| Cached completion | ~1 μs | Cache hit |

See `examples/benchmark.rs` for detailed performance measurements.

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

# Developer env

```
source <(./target/debug/examples/kubectl completion zsh)
```

