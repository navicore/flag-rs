# Flag-rs Roadmap: The Macro-First CLI Framework with Runtime Completions

## Vision Statement

Flag-rs aims to be **the definitive Rust CLI framework** by combining:
- **Clap's robustness** - Comprehensive argument parsing and validation
- **Cobra's dynamic features** - Runtime completions and ActiveHelp system
- **Rust's macro superpowers** - Clean, ergonomic API that eliminates boilerplate

**The 1.0 Goal**: One True Way™ - A macro-first API that makes building complex CLIs with dynamic completions as easy as deriving a struct.

## Current State vs Goals

### ✅ Already Implemented (Core Advantage)
- Dynamic runtime completions (inspired by Go Cobra)
- Shell completion generation (bash/zsh/fish)
- Zero external dependencies
- Basic command hierarchy and flag inheritance
- Builder pattern API

### 🎯 Major Gaps to Address
1. **Terminal infrastructure** - Width detection, text wrapping
2. **Argument validation** - Count validation, custom validators
3. **ActiveHelp system** - Contextual hints during completion (Cobra's killer feature)
4. **Command lifecycle hooks** - PreRun/PostRun for setup/teardown
5. **"Did you mean" suggestions** - Typo correction
6. **Macro API** - Ergonomic derive-based interface

## Development Phases

### Phase 1: Foundation (Core UX) - ✅ COMPLETED

#### 1.1 Terminal Infrastructure
- Terminal width detection using `$COLUMNS` + fallback to 80
- Text wrapping for help messages with proper word boundaries  
- Help formatting improvements with consistent alignment

#### 1.2 Argument Validation (Cobra-inspired)
```rust
pub enum ArgValidator {
    ExactArgs(usize),
    MinimumArgs(usize), 
    MaximumArgs(usize),
    RangeArgs(usize, usize),
    OnlyValidArgs,  // Must be in ValidArgs list
    Custom(Box<dyn Fn(&[String]) -> Result<()>>),
}

impl CommandBuilder {
    pub fn args(self, validator: ArgValidator) -> Self;
}
```

#### 1.3 "Did You Mean" Suggestions
```rust
impl Command {
    pub fn suggest_for(&mut self, suggestions: Vec<String>) -> &mut Self;
    pub fn suggestions_minimum_distance(&mut self, distance: usize) -> &mut Self;
    
    // Internal: Levenshtein distance calculation
    fn find_suggestions(&self, input: &str) -> Vec<String>;
}
```

**Deliverables**:
- [x] Terminal width detection
- [x] Help text wrapping
- [x] `ExactArgs`, `MinimumArgs`, `MaximumArgs`, `RangeArgs` validators
- [x] Basic suggestion system with Levenshtein distance
- [x] Integration tests for all validators

### Phase 2: ActiveHelp System (The Differentiator) - ✅ COMPLETED

#### 2.1 ActiveHelp Core
```rust
pub struct ActiveHelp {
    pub message: String,
    pub condition: Option<Box<dyn Fn(&Context) -> bool>>,
}

pub struct CompletionResult {
    pub values: Vec<String>,
    pub descriptions: HashMap<String, String>,
    pub active_help: Vec<ActiveHelp>,  // NEW!
}

impl CompletionResult {
    pub fn add_help<S: Into<String>>(mut self, help: S) -> Self;
    pub fn add_conditional_help<S, F>(mut self, help: S, condition: F) -> Self 
    where 
        S: Into<String>,
        F: Fn(&Context) -> bool + 'static;
}
```

#### 2.2 Shell Integration
- Modify bash/zsh/fish completion scripts to handle ActiveHelp markers
- Display help on double-TAB or when no completions available
- Proper formatting and color support per shell

#### 2.3 Example Usage
```rust
.arg_completion(|ctx, prefix| {
    let mut result = CompletionResult::new();
    result = result.extend(get_pod_names(prefix));
    
    if ctx.flag("namespace").is_none() {
        result = result.add_help("Tip: Use -n <namespace> to list pods in a specific namespace");
    }
    
    if prefix.is_empty() {
        result = result.add_help("Use TAB to see available pods, or type a partial name");
    }
    
    Ok(result)
})
```

**Deliverables**:
- [x] ActiveHelp data structures
- [x] Shell script modifications for all supported shells
- [x] ActiveHelp display logic (special prefix formatting for each shell)
- [x] Examples demonstrating contextual help
- [x] Integration tests for ActiveHelp functionality

### Phase 3: Command Lifecycle Hooks (Cobra-style) ✅

#### 3.1 Command Lifecycle Hooks (Cobra-style)
```rust
pub struct Command {
    persistent_pre_run: Option<Box<dyn Fn(&mut Context) -> Result<()>>>,
    pre_run: Option<Box<dyn Fn(&mut Context) -> Result<()>>>,
    post_run: Option<Box<dyn Fn(&mut Context) -> Result<()>>>,
    persistent_post_run: Option<Box<dyn Fn(&mut Context) -> Result<()>>>,
}

impl CommandBuilder {
    pub fn persistent_pre_run<F>(self, f: F) -> Self 
    where F: Fn(&mut Context) -> Result<()> + 'static;
    
    pub fn pre_run<F>(self, f: F) -> Self 
    where F: Fn(&mut Context) -> Result<()> + 'static;
    
    pub fn post_run<F>(self, f: F) -> Self 
    where F: Fn(&mut Context) -> Result<()> + 'static;
    
    pub fn persistent_post_run<F>(self, f: F) -> Self 
    where F: Fn(&mut Context) -> Result<()> + 'static;
}
```

#### 3.2 Execution Order
```
PersistentPreRun (parent → child)
PreRun (current command only)  
Run (main command)
PostRun (current command only)
PersistentPostRun (child → parent)
```

**Deliverables**:
- [x] Lifecycle hook infrastructure
- [x] Proper execution order with error handling
- [x] Examples showing setup/teardown patterns
- [x] Integration with existing command execution

### Phase 4: Enhanced Help & UX - 1-2 weeks

#### 4.1 Better Error Messages
```rust
pub enum Error {
    CommandNotFound { 
        command: String, 
        suggestions: Vec<String> 
    },
    ArgumentValidation {
        message: String,
        expected: String,
        received: usize,
    },
    // Enhanced existing variants...
}
```

#### 4.2 Command Groups (Optional)
```rust
pub struct CommandGroup {
    pub id: String,
    pub title: String,
}

impl CommandBuilder {
    pub fn group_id(self, group_id: &str) -> Self;
}
```

**Deliverables**:
- [ ] Enhanced error messages with suggestions
- [ ] Better help formatting
- [ ] Optional command grouping support

### Phase 5: Advanced Flag Features - 1-2 weeks

#### 5.1 Enhanced Flag Types
```rust
pub enum FlagType {
    // Existing types...
    StringArray,  // Multiple values: --tag=a --tag=b
    Choice(Vec<String>), // Must be one of these values  
    Range(i64, i64),     // Numeric range validation
    File,         // File path validation
    Directory,    // Directory path validation
}
```

#### 5.2 Flag Constraints
```rust
pub enum FlagConstraint {
    RequiredIf(String),      // Required if another flag is set
    ConflictsWith(Vec<String>), // Mutually exclusive
    Requires(Vec<String>),   // Must be used with these flags
}

impl Flag {
    pub fn constraint(mut self, constraint: FlagConstraint) -> Self;
}
```

**Deliverables**:
- [ ] Enhanced flag types with validation
- [ ] Flag constraint system
- [ ] Comprehensive validation error messages

### Phase 6: Performance & Polish - 1 week

#### 6.1 Performance Optimizations
- Completion caching for expensive operations
- Timeout handling for slow completions
- Memory usage optimization for large CLIs

#### 6.2 Testing & Documentation
- Integration tests for all shell completion scripts
- Performance benchmarks vs clap
- Comprehensive documentation updates

**Deliverables**:
- [ ] Performance optimizations
- [ ] Comprehensive test suite
- [ ] Benchmarks vs clap
- [ ] Updated documentation

### Phase 7: Macro MVP - 2-3 weeks

#### 7.1 Foundation for Macros
Design builder API to be macro-friendly:

```rust
// Rich context for completion functions
pub struct CompletionContext<'a> {
    pub command_path: &'a [String],
    pub flags: &'a HashMap<String, String>,
    pub args: &'a [String],
    pub current_arg_index: usize,
    pub current_arg_name: Option<&'a str>,
}

// Metadata-rich hooks
pub struct HookMetadata {
    pub source: String,  // "derive_macro", "user_defined"
    pub command_path: Vec<String>,
}

// Rich validation errors
pub struct ValidationError {
    pub field_name: Option<String>,
    pub struct_name: Option<String>,
    pub message: String,
    pub suggestions: Vec<String>,
}
```

#### 7.2 Basic Macro Implementation
```rust
#[derive(Cli)]
struct App {
    #[arg(short, long)]
    verbose: bool,
    
    #[command]
    cmd: Commands,
}

#[derive(Commands)]
enum Commands {
    Get(GetCmd),
    Delete(DeleteCmd),
}

#[derive(Args)]
struct GetCmd {
    #[arg()]
    resource: String,
}
```

**Deliverables**:
- [ ] Basic `#[derive(Cli)]`, `#[derive(Commands)]`, `#[derive(Args)]`
- [ ] Foundation for completion macros
- [ ] Macro-friendly builder API design
- [ ] Basic examples and tests

### Phase 8: Completion Macros - 2 weeks

#### 8.1 Completion Integration
```rust
#[derive(Args)]
struct Get {
    #[arg()]
    #[complete(resource_types)]  // Function-based completion
    resource: String,
    
    #[arg()]
    #[complete(resource_names)]  // Context-aware completion
    name: Option<String>,
}

fn resource_names(ctx: &CompletionContext, prefix: &str) -> Result<CompletionResult> {
    let resource = ctx.get_arg("resource")?;
    match resource.as_str() {
        "pods" => get_pod_names(prefix),
        "services" => get_service_names(prefix),
        _ => CompletionResult::new().add_help("Unknown resource type"),
    }
}
```

**Deliverables**:
- [ ] `#[complete(function)]` macro attribute
- [ ] Context-aware completion functions
- [ ] Integration with existing completion system

### Phase 9: Advanced Macro Features - 3-4 weeks

#### 9.1 Built-in Validators
```rust
#[derive(Args)]
struct Deploy {
    #[arg()]
    #[validate(file_exists)]
    #[validate(yaml_format)]
    #[validate(k8s_resource)]  // Chain validators
    file: PathBuf,
}
```

#### 9.2 ActiveHelp Macros
```rust
#[derive(Args)]
struct GetPods {
    #[arg()]
    #[complete(pod_names)]
    #[help_hint("Use TAB to see available pods")]
    name: Option<String>,
}
```

#### 9.3 Lifecycle Hook Macros
```rust
#[derive(Cli)]
#[cli(pre_run = setup_logging)]
struct App {
    // ...
}
```

**Deliverables**:
- [ ] Built-in validator library
- [ ] ActiveHelp macro integration
- [ ] Lifecycle hook macros
- [ ] Comprehensive macro documentation

## The 1.0 Vision

The ultimate goal is this level of simplicity for complex CLIs:

```rust
#[derive(Cli)]
#[cli(name = "kubectl", version = "1.0.0")]
struct Kubectl {
    #[arg(short = 'n', long, default = "default")]
    #[complete(namespaces)]
    namespace: String,

    #[command]
    cmd: Commands,
}

#[derive(Commands)]
enum Commands {
    /// Get resources
    Get {
        #[arg()]
        #[complete(resource_types)]
        resource: String,
        
        #[arg()]
        #[complete(resource_names)]
        name: Option<String>,
    },
    
    /// Delete resources  
    Delete {
        #[arg()]
        #[complete(resource_types)]
        resource: String,
        
        #[arg(required = true)]
        #[complete(resource_names)]
        name: String,
    },
}

fn main() {
    Kubectl::parse().execute().unwrap();
}
```

## Success Metrics

### Feature Parity
- [ ] All core clap features implemented
- [ ] Argument validation as comprehensive as clap
- [ ] Help system as polished as clap

### Unique Value Propositions
- [ ] Runtime completions working seamlessly
- [ ] ActiveHelp providing contextual assistance
- [ ] Macro API eliminating boilerplate

### Performance & Usability
- [ ] Completion latency < 100ms for common cases
- [ ] Zero dependencies maintained
- [ ] Migration path from clap documented

### Developer Experience
- [ ] One True Way™ - Clear best practice
- [ ] Comprehensive examples (kubectl-like tools)
- [ ] Excellent error messages and debugging

## Positioning Statement

**Flag-rs 1.0**: The macro-first CLI framework with runtime completions

- **vs Clap**: "All of clap's robustness + runtime completions + ActiveHelp + ergonomic macros"
- **vs Cobra**: "Cobra's dynamic features in Rust with zero dependencies and macro ergonomics"

## Getting Started

1. **Phase 1** focus: Terminal infrastructure and argument validation (high impact, foundational)
2. **Phase 2** priority: ActiveHelp system (unique differentiator)
3. **Builder API first**: Ensure solid foundation before macro development
4. **Iterative feedback**: Test each phase with real kubectl examples

The goal is to make flag-rs the obvious choice for any developer building a CLI tool that needs dynamic behavior - whether that's Kubernetes tools, cloud CLIs, database tools, or development utilities.