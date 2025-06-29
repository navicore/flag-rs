# Flag-rs Macro Design Document

## Phase 7: Macro Implementation Strategy

### Overview This document captures the design decisions and implementation
strategy for the flag-rs macro system, building on the improved builder API from
Phases 1-6.

### Foundation Work Completed The builder API improvements in Phase 6 were
specifically designed to support macro generation:

1. **Type-specific constructors**: `Flag::bool()`, `Flag::int()`, etc. map
   directly to macro type inference
2. **Inline completions**: `Flag::completion()` allows macros to generate
   completion closures
3. **Bulk operations**: `flags()` and `subcommands()` allow macros to generate
   collections efficiently
4. **Type-safe access**: `ctx.flag_bool()`, etc. provide the runtime API that
   macros will generate calls to

### Macro Architecture

#### Core Derive Macros ```rust #[derive(Cli)]           // Main application
entry point #[derive(Commands)]      // Enum of subcommands #[derive(Args)]
// Struct of arguments/flags ```

#### Design Principles
1. **Zero-cost abstraction**: Macros should generate the same code a user would
   write manually
2. **Progressive disclosure**: Simple use cases should be simple, complex ones
   possible
3. **Type safety**: Leverage Rust's type system for compile-time guarantees
4. **Completion-first**: Make dynamic completions as easy as static ones

### Implementation Plan

#### Step 1: Basic Derive Infrastructure ```rust // flag-rs-derive/src/lib.rs
#[proc_macro_derive(Cli, attributes(cli))] pub fn derive_cli(input: TokenStream)
-> TokenStream { ... }

#[proc_macro_derive(Commands, attributes(command))] pub fn
derive_commands(input: TokenStream) -> TokenStream { ... }

#[proc_macro_derive(Args, attributes(arg))] pub fn derive_args(input:
TokenStream) -> TokenStream { ... } ```

#### Step 2: Type Mapping Strategy ```rust // Rust type -> Flag constructor
mapping bool                -> Flag::bool() i8/i16/i32/i64     -> Flag::int()
f32/f64            -> Flag::float() String/&str        -> Flag::string()
Vec<String>        -> Flag::string_slice() PathBuf            -> Flag::file() or
Flag::directory() Option<T>          -> optional flag T                  ->
required flag ```

#### Step 3: Attribute Design ```rust #[derive(Args)] struct Config { /// Enable
verbose output #[arg(short = 'v', long)] verbose: bool,

    /// Server port #[arg(short, long, default = 8080)] port: i32,

    /// Config file path #[arg(long, value_name = "FILE")]
    #[complete(config_files)]  // Function name for completion config:
    Option<PathBuf>,

    /// Environment to deploy to #[arg(long, value_enum)] environment:
    Environment,  // Enum generates Choice flag }

#[derive(ValueEnum)] enum Environment { Dev, Staging, Prod, } ```

#### Step 4: Completion Integration ```rust // Completion functions referenced
by name fn config_files(ctx: &Context, prefix: &str) -> Result<CompletionResult>
{ // Implementation }

// Or inline with closure syntax #[complete(|ctx, prefix| { // Inline
implementation })] ```

#### Step 5: Generated Code Pattern The macro should generate code that uses our
improved builder API:

```rust // From: #[derive(Cli)] struct App { ... } // Generates: impl App { pub
fn command() -> Command { CommandBuilder::new("app") .flags(vec![
Flag::bool("verbose") .short('v') .usage("Enable verbose output"),
Flag::int("port") .short('p') .default_int(8080) .usage("Server port"), // ...
etc ]) .run(|ctx| { let app = App::from_context(ctx)?; app.run() }) .build() }

    fn from_context(ctx: &Context) -> Result<Self> { Ok(Self { verbose:
    ctx.flag_bool_or("verbose", false), port: ctx.flag_int_or("port", 8080), //
    ... etc }) } } ```

### Key Technical Challenges

1. **CRITICAL: Runtime Completions**: Unlike clap, completions MUST remain dynamic
   - Completions are generated at runtime when user presses TAB
   - Completion functions can query live systems (k8s API, databases, etc.)
   - This is flag-rs's killer feature - preserve it in macro design
   - Macros generate completion function calls, NOT static completion lists
2. **Completion function resolution**: Need to resolve function names at compile
   time
3. **Error messages**: Provide clear, actionable error messages for macro users
4. **Hygiene**: Ensure generated code doesn't conflict with user code
5. **Feature flags**: Support optional features (e.g., async support)

### Testing Strategy

1. **Macro expansion tests**: Verify correct code generation
2. **Compile fail tests**: Ensure good error messages
3. **Integration tests**: Full CLI apps using macros
4. **Example coverage**: Examples for every macro feature

### No Migration Path Needed

**Important**: The builder API is for internal use and early feedback only. When macros are released:
- The macro API becomes the primary (only) public API
- Builder API becomes internal implementation detail
- No migration path needed - early users understand this is pre-release
- Free to break/change builder API to best support macro generation

### Success Metrics

1. **API surface reduction**: 80% less code for typical CLI
2. **Compilation time**: < 2s overhead for macro processing
3. **Error quality**: All macro errors should suggest fixes
4. **Feature parity**: Everything possible with builder API possible with macros

### Future Extensions

1. **Validation macros**: `#[validate(range(1, 100))]`
2. **Async support**: `#[arg(async_complete = fetch_from_api)]`
3. **Subcommand modules**: `#[commands(path = "src/commands")]`
4. **Custom derive helpers**: User-defined arg types

### Notes for Implementation

- Start with the simplest possible macro (bool flag only)
- Add features incrementally with tests for each
- Prioritize error messages from day one
- Keep generated code readable for debugging
- Document macro expansion for users

This design leverages all the builder API improvements we've made and provides a
clear path to the "One True Way™" macro API that will make flag-rs the
definitive Rust CLI framework.
