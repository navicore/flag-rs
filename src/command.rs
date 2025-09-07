//! Command execution and management
//!
//! This module provides the core [`Command`] struct and [`CommandBuilder`] for creating
//! CLI applications with subcommands, flags, and dynamic completions.

use crate::completion::{CompletionFunc, CompletionResult};
use crate::completion_format::CompletionFormat;
use crate::context::Context;
use crate::error::{Error, Result};
use crate::flag::{Flag, FlagConstraint, FlagType, FlagValue};
use crate::suggestion::{DEFAULT_SUGGESTION_DISTANCE, find_suggestions};
use crate::terminal::{format_help_entry, get_terminal_width, wrap_text_to_terminal};
use crate::validator::ArgValidator;
use std::collections::{HashMap, HashSet};

/// Type alias for the function that executes when a command runs
pub type RunFunc = Box<dyn Fn(&mut Context) -> Result<()> + Send + Sync>;

/// Type alias for lifecycle hook functions
pub type HookFunc = Box<dyn Fn(&mut Context) -> Result<()> + Send + Sync>;

/// Represents a command in the CLI application
///
/// Commands can have:
/// - Subcommands for nested command structures
/// - Flags that modify behavior
/// - A run function that executes the command logic
/// - Dynamic completion functions for arguments and flags
/// - Help text and aliases
///
/// # Examples
///
/// ```rust
/// use flag_rs::{Command, CommandBuilder, Context};
///
/// // Using the builder pattern (recommended)
/// let cmd = CommandBuilder::new("serve")
///     .short("Start the web server")
///     .run(|ctx| {
///         println!("Server starting...");
///         Ok(())
///     })
///     .build();
///
/// // Direct construction
/// let mut cmd = Command::new("serve");
/// ```
pub struct Command {
    name: String,
    aliases: Vec<String>,
    short: String,
    long: String,
    examples: Vec<String>,
    group_id: Option<String>,
    subcommands: HashMap<String, Self>,
    flags: HashMap<String, Flag>,
    run: Option<RunFunc>,
    parent: Option<*mut Self>,
    arg_completions: Option<CompletionFunc>,
    flag_completions: HashMap<String, CompletionFunc>,
    arg_validator: Option<ArgValidator>,
    suggestions_enabled: bool,
    suggestion_distance: usize,
    // Lifecycle hooks
    persistent_pre_run: Option<HookFunc>,
    pre_run: Option<HookFunc>,
    post_run: Option<HookFunc>,
    persistent_post_run: Option<HookFunc>,
}

unsafe impl Send for Command {}
unsafe impl Sync for Command {}
/// Collects all available flags with their descriptions for completion
fn collect_all_flags_with_descriptions(
    current: &Command,
    result: &mut CompletionResult,
    prefix: &str,
) {
    // Add current command's flags
    for (flag_name, flag) in &current.flags {
        if flag_name.starts_with(prefix) {
            let formatted_flag = format!("--{flag_name}");
            *result = result
                .clone()
                .add_with_description(formatted_flag, flag.usage.clone());
        }
    }

    // Add parent flags
    if let Some(parent) = current.parent {
        unsafe {
            collect_all_flags_with_descriptions(&*parent, result, prefix);
        }
    }
}

impl Command {
    /// Creates a new command with the given name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::Command;
    ///
    /// let cmd = Command::new("myapp");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aliases: Vec::new(),
            short: String::new(),
            long: String::new(),
            examples: Vec::new(),
            group_id: None,
            subcommands: HashMap::new(),
            flags: HashMap::new(),
            run: None,
            parent: None,
            arg_completions: None,
            flag_completions: HashMap::new(),
            arg_validator: None,
            suggestions_enabled: true,
            suggestion_distance: DEFAULT_SUGGESTION_DISTANCE,
            persistent_pre_run: None,
            pre_run: None,
            post_run: None,
            persistent_post_run: None,
        }
    }

    /// Returns the command name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the short description
    pub fn short(&self) -> &str {
        &self.short
    }

    /// Returns the long description
    pub fn long(&self) -> &str {
        &self.long
    }

    /// Returns a reference to all subcommands
    pub fn subcommands(&self) -> &HashMap<String, Self> {
        &self.subcommands
    }

    /// Returns a reference to all flags
    pub fn flags(&self) -> &HashMap<String, Flag> {
        &self.flags
    }

    /// Finds a subcommand by name or alias
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use flag_rs::{Command, CommandBuilder};
    /// let mut root = Command::new("app");
    /// let sub = CommandBuilder::new("server")
    ///     .aliases(vec!["serve", "s"])
    ///     .build();
    /// root.add_command(sub);
    ///
    /// assert!(root.find_subcommand("server").is_some());
    /// assert!(root.find_subcommand("serve").is_some());
    /// assert!(root.find_subcommand("s").is_some());
    /// ```
    pub fn find_subcommand(&self, name: &str) -> Option<&Self> {
        self.subcommands.get(name).or_else(|| {
            self.subcommands
                .values()
                .find(|cmd| cmd.aliases.contains(&name.to_string()))
        })
    }

    /// Finds a mutable reference to a subcommand by name or alias
    pub fn find_subcommand_mut(&mut self, name: &str) -> Option<&mut Self> {
        let name_string = name.to_string();
        if self.subcommands.contains_key(name) {
            self.subcommands.get_mut(name)
        } else {
            self.subcommands
                .values_mut()
                .find(|cmd| cmd.aliases.contains(&name_string))
        }
    }

    /// Adds a subcommand to this command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{Command, CommandBuilder};
    ///
    /// let mut root = Command::new("myapp");
    /// let serve = CommandBuilder::new("serve")
    ///     .short("Start the server")
    ///     .build();
    ///
    /// root.add_command(serve);
    /// ```
    pub fn add_command(&mut self, mut cmd: Self) {
        cmd.parent = Some(self as *mut Self);
        self.subcommands.insert(cmd.name.clone(), cmd);
    }

    /// Executes the command with the given arguments
    ///
    /// This is the main entry point for running your CLI application.
    /// It handles:
    /// - Shell completion requests
    /// - Flag parsing
    /// - Subcommand routing
    /// - Execution of the appropriate run function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let app = CommandBuilder::new("myapp")
    ///     .run(|ctx| {
    ///         println!("Hello from myapp!");
    ///         Ok(())
    ///     })
    ///     .build();
    ///
    /// // In main():
    /// // let args: Vec<String> = std::env::args().skip(1).collect();
    /// // if let Err(e) = app.execute(args) {
    /// //     eprintln!("Error: {}", e);
    /// //     std::process::exit(1);
    /// // }
    /// ```
    pub fn execute(&self, args: Vec<String>) -> Result<()> {
        // Check if we're in completion mode
        if let Ok(_shell) = std::env::var(format!("{}_COMPLETE", self.name.to_uppercase())) {
            // Disable colors during completion to avoid terminal rendering issues
            unsafe { std::env::set_var("NO_COLOR", "1") };

            match self.handle_completion_request(&args) {
                Ok(suggestions) => {
                    for suggestion in suggestions {
                        println!("{suggestion}");
                    }
                    return Ok(());
                }
                Err(e) => {
                    // Don't write to stderr during completion - it can mess up the terminal
                    return Err(e);
                }
            }
        }

        let mut ctx = Context::new(args);
        self.execute_with_context(&mut ctx)
    }

    /// Executes the command with an existing context
    ///
    /// This method is useful when you need to provide pre-configured context
    /// or when implementing custom command routing.
    pub fn execute_with_context(&self, ctx: &mut Context) -> Result<()> {
        // Call the internal method with an empty hook chain
        self.execute_with_context_and_hooks(ctx, &mut Vec::new())
    }

    /// Internal method that executes the command while collecting parent hooks
    fn execute_with_context_and_hooks<'a>(
        &'a self,
        ctx: &mut Context,
        parent_hooks: &mut Vec<(&'a Option<HookFunc>, &'a Option<HookFunc>)>,
    ) -> Result<()> {
        let args = ctx.args().to_vec();

        // Parse flags first, before checking for empty args
        let (flags, remaining_args) = self.parse_flags(&args)?;

        *ctx.args_mut() = remaining_args;

        // Check if we have a subcommand first
        if let Some(subcommand_name) = ctx.args().first() {
            if let Some(subcommand) = self.find_subcommand(subcommand_name) {
                // If help flag is present, show help for the subcommand
                if flags.contains_key("help") {
                    subcommand.print_help();
                    return Ok(());
                }

                // Validate flags before setting them
                self.validate_flags(&flags)?;

                // Set flags and execute subcommand
                for (name, value) in flags {
                    ctx.set_flag(name, value);
                }

                // Add our persistent hooks to the chain for subcommands
                parent_hooks.push((&self.persistent_pre_run, &self.persistent_post_run));

                ctx.args_mut().remove(0);
                return subcommand.execute_with_context_and_hooks(ctx, parent_hooks);
            }
        }

        // No subcommand found, check for help at this level
        if flags.contains_key("help") {
            self.print_help();
            return Ok(());
        }

        // Validate flags before setting them
        self.validate_flags(&flags)?;

        // Set flags
        for (name, value) in flags {
            ctx.set_flag(name, value);
        }

        // No subcommand found, try to run this command's function
        if let Some(ref run) = self.run {
            // Validate arguments before running
            if let Some(ref validator) = self.arg_validator {
                validator.validate(ctx.args())?;
            }
            self.execute_with_parent_hooks(ctx, run, parent_hooks)
        } else if ctx.args().is_empty() {
            // No args and no run function - show help
            Err(Error::SubcommandRequired(self.name.clone()))
        } else {
            let unknown_command = ctx.args().first().unwrap_or(&String::new()).clone();
            let suggestions = if self.suggestions_enabled {
                self.find_command_suggestions(&unknown_command)
            } else {
                Vec::new()
            };

            Err(Error::CommandNotFound {
                command: unknown_command,
                suggestions,
            })
        }
    }

    fn parse_flags(&self, args: &[String]) -> Result<(HashMap<String, String>, Vec<String>)> {
        let mut flags = HashMap::new();
        let mut remaining = Vec::new();
        let mut i = 0;

        while i < args.len() {
            let arg = &args[i];

            if arg == "--" {
                remaining.extend_from_slice(&args[i + 1..]);
                break;
            } else if arg.starts_with("--") {
                let flag_name = arg.trim_start_matches("--");

                // Special handling for help
                if flag_name == "help" {
                    flags.insert("help".to_string(), "true".to_string());
                } else if let Some((name, value)) = flag_name.split_once('=') {
                    // Validate the flag value
                    if let Some(flag) = self.find_flag(name) {
                        flag.parse_value(value)?;
                    }
                    flags.insert(name.to_string(), value.to_string());
                } else if let Some(flag) = self.find_flag(flag_name) {
                    if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                        let value = &args[i + 1];
                        // Validate the flag value
                        flag.parse_value(value)?;
                        flags.insert(flag_name.to_string(), value.clone());
                        i += 1;
                    } else {
                        flags.insert(flag_name.to_string(), "true".to_string());
                    }
                } else {
                    // Unknown flag - might belong to a subcommand
                    remaining.push(arg.clone());
                }
            } else if arg.starts_with('-') && arg.len() > 1 {
                let short_flags = arg.trim_start_matches('-');
                let chars: Vec<char> = short_flags.chars().collect();

                for (idx, ch) in chars.iter().enumerate() {
                    // Special handling for -h as help
                    if *ch == 'h' {
                        flags.insert("help".to_string(), "true".to_string());
                    } else if let Some(flag) = self.find_flag_by_short(*ch) {
                        // If this is the last char and the flag takes a value
                        if idx == chars.len() - 1
                            && i + 1 < args.len()
                            && !args[i + 1].starts_with('-')
                        {
                            let value = &args[i + 1];
                            // Validate the flag value
                            flag.parse_value(value)?;
                            flags.insert(flag.name.clone(), value.clone());
                            i += 1;
                        } else {
                            flags.insert(flag.name.clone(), "true".to_string());
                        }
                    } else {
                        // Unknown short flag - might belong to a subcommand
                        remaining.push(format!("-{}", chars[idx..].iter().collect::<String>()));
                        break;
                    }
                }
            } else {
                remaining.push(arg.clone());
            }

            i += 1;
        }

        Ok((flags, remaining))
    }

    /// Sets the argument completion function for this command
    ///
    /// The completion function is called when the user presses TAB to complete
    /// command arguments. It receives the current context and the prefix to complete.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{Command, CompletionResult};
    ///
    /// let mut cmd = Command::new("get");
    /// cmd.set_arg_completion(|ctx, prefix| {
    ///     let items = vec!["users", "posts", "comments"];
    ///     Ok(CompletionResult::new().extend(
    ///         items.into_iter()
    ///             .filter(|i| i.starts_with(prefix))
    ///             .map(String::from)
    ///     ))
    /// });
    /// ```
    pub fn set_arg_completion<F>(&mut self, f: F)
    where
        F: Fn(&Context, &str) -> Result<CompletionResult> + Send + Sync + 'static,
    {
        self.arg_completions = Some(Box::new(f));
    }

    /// Sets the completion function for a specific flag
    ///
    /// This allows dynamic completion of flag values based on runtime state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{Command, CompletionResult};
    ///
    /// let mut cmd = Command::new("deploy");
    /// cmd.set_flag_completion("environment", |ctx, prefix| {
    ///     let envs = vec!["dev", "staging", "production"];
    ///     Ok(CompletionResult::new().extend(
    ///         envs.into_iter()
    ///             .filter(|e| e.starts_with(prefix))
    ///             .map(String::from)
    ///     ))
    /// });
    /// ```
    pub fn set_flag_completion<F>(&mut self, flag_name: impl Into<String>, f: F)
    where
        F: Fn(&Context, &str) -> Result<CompletionResult> + Send + Sync + 'static,
    {
        self.flag_completions.insert(flag_name.into(), Box::new(f));
    }

    /// Gets completion suggestions for the current context
    ///
    /// This method is primarily used internally by the shell completion system.
    pub fn get_completions(
        &self,
        ctx: &Context,
        to_complete: &str,
        completing_flag: Option<&str>,
    ) -> Result<CompletionResult> {
        if let Some(flag_name) = completing_flag {
            if let Some(completion_func) = self.flag_completions.get(flag_name) {
                return completion_func(ctx, to_complete);
            }
        } else if let Some(ref completion_func) = self.arg_completions {
            return completion_func(ctx, to_complete);
        }

        Ok(CompletionResult::new())
    }

    fn find_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.get(name).or_else(|| {
            self.parent
                .and_then(|parent| unsafe { (*parent).find_flag(name) })
        })
    }

    fn find_flag_by_short(&self, short: char) -> Option<&Flag> {
        self.flags
            .values()
            .find(|f| f.short == Some(short))
            .or_else(|| {
                self.parent
                    .and_then(|parent| unsafe { (*parent).find_flag_by_short(short) })
            })
    }

    /// Validates all flags including required flags and constraints
    fn validate_flags(&self, provided_flags: &HashMap<String, String>) -> Result<()> {
        let provided_flag_names: HashSet<String> = provided_flags.keys().cloned().collect();

        // Check required flags
        for (flag_name, flag) in &self.flags {
            if flag.required && !provided_flag_names.contains(flag_name) {
                return Err(Error::flag_parsing_with_suggestions(
                    format!("Required flag '--{flag_name}' not provided"),
                    flag_name.to_string(),
                    vec![format!("add --{flag_name} <value>")],
                ));
            }
        }

        // TODO: Fix unsafe parent flag validation
        // Check parent flags if any
        // if let Some(parent) = self.parent {
        //     unsafe {
        //         for (flag_name, flag) in &(*parent).flags {
        //             if flag.required && !provided_flag_names.contains(flag_name) {
        //                 return Err(Error::flag_parsing_with_suggestions(
        //                     format!("Required flag '--{flag_name}' not provided"),
        //                     flag_name.to_string(),
        //                     vec![format!("add --{flag_name} <value>")],
        //                 ));
        //             }
        //         }
        //     }
        // }

        // Validate constraints for all flags
        for (flag_name, flag) in &self.flags {
            flag.validate_constraints(flag_name, &provided_flag_names)?;
        }

        // TODO: Fix unsafe parent flag constraint validation
        // The current approach with raw pointers can lead to undefined behavior
        // when the parent Command is moved or when accessing heap-allocated data
        // through the pointer (like Vec<FlagConstraint>).
        //
        // Validate parent flag constraints
        // if let Some(parent) = self.parent {
        //     unsafe {
        //         for (flag_name, flag) in &(*parent).flags {
        //             flag.validate_constraints(flag_name, &provided_flag_names)?;
        //         }
        //     }
        // }

        Ok(())
    }

    /// Executes the command with lifecycle hooks including parent hooks
    fn execute_with_parent_hooks(
        &self,
        ctx: &mut Context,
        run: &RunFunc,
        parent_hooks: &[(&Option<HookFunc>, &Option<HookFunc>)],
    ) -> Result<()> {
        // Execute parent persistent pre-run hooks (from root to immediate parent)
        for (pre_hook, _) in parent_hooks {
            if let Some(hook) = pre_hook {
                hook(ctx)?;
            }
        }

        // Execute own persistent pre-run hook if present
        if let Some(ref hook) = self.persistent_pre_run {
            hook(ctx)?;
        }

        // Execute pre-run hook if present
        if let Some(ref pre_run) = self.pre_run {
            pre_run(ctx)?;
        }

        // Execute the main run function
        let result = run(ctx);

        // Execute post-run hook if present, but preserve the original error
        let post_run_result = if let Some(ref post_run) = self.post_run {
            match result {
                Ok(()) => post_run(ctx),
                Err(e) => {
                    // Try to run post-run even if main failed, but return original error
                    let _ = post_run(ctx);
                    Err(e)
                }
            }
        } else {
            result
        };

        // Execute own persistent post-run hook if present
        let persistent_result = if let Some(ref hook) = self.persistent_post_run {
            let result = hook(ctx);
            match post_run_result {
                Ok(()) => result,
                Err(e) => {
                    // Try to run persistent post-run even if post-run failed
                    let _ = result;
                    Err(e)
                }
            }
        } else {
            post_run_result
        };

        // Execute parent persistent post-run hooks (from immediate parent to root)
        let mut final_result = persistent_result;
        for (_, post_hook) in parent_hooks.iter().rev() {
            if let Some(hook) = post_hook {
                match final_result {
                    Ok(()) => final_result = hook(ctx),
                    Err(e) => {
                        // Try to run parent post-run even if child failed
                        let _ = hook(ctx);
                        final_result = Err(e);
                    }
                }
            }
        }

        final_result
    }

    /// Prints the help message for this command
    ///
    /// The help message includes:
    /// - Command description
    /// - Usage information
    /// - Available subcommands
    /// - Local and global flags
    ///
    /// Help text is automatically colored when outputting to a TTY.
    #[allow(clippy::cognitive_complexity)]
    pub fn print_help(&self) {
        use crate::color;

        // Print description with text wrapping
        if !self.long.is_empty() {
            println!("{}", wrap_text_to_terminal(&self.long, None));
            println!();
        } else if !self.short.is_empty() {
            println!("{}", wrap_text_to_terminal(&self.short, None));
            println!();
        }

        // Print usage line
        print!("{}:\n  {}", color::bold("Usage"), self.name);
        if !self.flags.is_empty() {
            print!(" {}", color::yellow("[flags]"));
        }
        if !self.subcommands.is_empty() {
            print!(" {}", color::yellow("[command]"));
        }

        // Show if command requires args
        if let Some(validator) = &self.arg_validator {
            match validator {
                ArgValidator::MinimumArgs(n) if n > &0 => {
                    print!(" {}", color::yellow("<args>"));
                }
                ArgValidator::ExactArgs(n) if n > &0 => {
                    let arg_str = if n == &1 { "<arg>" } else { "<args>" };
                    print!(" {}", color::yellow(arg_str));
                }
                ArgValidator::RangeArgs(min, _) if min > &0 => {
                    print!(" {}", color::yellow("<args>"));
                }
                _ => {}
            }
        }
        println!("\n");

        // Print available commands
        if !self.subcommands.is_empty() {
            let mut commands: Vec<_> = self.subcommands.values().collect();
            commands.sort_by_key(|cmd| &cmd.name);

            // Group commands by their group_id
            let mut grouped: std::collections::BTreeMap<Option<String>, Vec<&Self>> =
                std::collections::BTreeMap::new();
            for cmd in commands {
                grouped.entry(cmd.group_id.clone()).or_default().push(cmd);
            }

            let terminal_width = get_terminal_width();
            let left_column_width = 24;

            // Print commands without groups first
            if let Some(ungrouped) = grouped.get(&None) {
                println!("{}:", color::bold("Available Commands"));
                for cmd in ungrouped {
                    // Build command name with aliases
                    let mut name_with_aliases = color::green(&cmd.name);
                    if !cmd.aliases.is_empty() {
                        let aliases = cmd.aliases.join(", ");
                        name_with_aliases = format!(
                            "{} {}",
                            name_with_aliases,
                            color::dim(&format!("({aliases})"))
                        );
                    }

                    let formatted = format_help_entry(
                        &format!("  {name_with_aliases}"),
                        &cmd.short,
                        left_column_width + 2, // account for the "  " prefix
                        terminal_width,
                    );
                    println!("{formatted}");
                }
                println!();
            }

            // Print grouped commands
            for (group_id, cmds) in grouped {
                if let Some(group) = group_id {
                    println!("{}:", color::bold(&group));
                    for cmd in cmds {
                        // Build command name with aliases
                        let mut name_with_aliases = color::green(&cmd.name);
                        if !cmd.aliases.is_empty() {
                            let aliases = cmd.aliases.join(", ");
                            name_with_aliases = format!(
                                "{} {}",
                                name_with_aliases,
                                color::dim(&format!("({aliases})"))
                            );
                        }

                        let formatted = format_help_entry(
                            &format!("  {name_with_aliases}"),
                            &cmd.short,
                            left_column_width + 2, // account for the "  " prefix
                            terminal_width,
                        );
                        println!("{formatted}");
                    }
                    println!();
                }
            }
        }

        // Print flags
        if !self.flags.is_empty() || self.parent.is_some() {
            // Separate required and optional flags
            let mut required_flags: Vec<_> = self.flags.values().filter(|f| f.required).collect();
            let mut optional_flags: Vec<_> = self.flags.values().filter(|f| !f.required).collect();

            required_flags.sort_by_key(|f| &f.name);
            optional_flags.sort_by_key(|f| &f.name);

            // Print required flags first
            if !required_flags.is_empty() {
                println!("{} {}:", color::bold("Required Flags"), color::red("*"));
                for flag in required_flags {
                    Self::print_flag(flag);
                }
                if !optional_flags.is_empty() {
                    println!();
                }
            }

            // Print optional flags
            if !optional_flags.is_empty() {
                println!("{}:", color::bold("Flags"));
                for flag in optional_flags {
                    Self::print_flag(flag);
                }
            }
        }

        // Print global flags from parent
        if let Some(parent) = self.parent {
            unsafe {
                let parent_flags = &(*parent).flags;
                if !parent_flags.is_empty() {
                    println!("\n{}:", color::bold("Global Flags"));
                    let mut global_flags: Vec<_> = parent_flags.values().collect();
                    global_flags.sort_by_key(|f| &f.name);

                    for flag in global_flags {
                        Self::print_flag(flag);
                    }
                }
            }
        }

        // Print examples if available
        if !self.examples.is_empty() {
            println!("{}:", color::bold("Examples"));
            for example in &self.examples {
                println!("  {}", color::dim(example));
            }
            println!();
        }

        // Print help about help
        println!(
            "Use \"{} {} --help\" for more information about a command.",
            self.name,
            color::yellow("[command]")
        );
    }

    fn print_flag(flag: &Flag) {
        use crate::color;
        use std::fmt::Write;

        let short = flag
            .short
            .map_or_else(|| "    ".to_string(), |s| format!("-{s}, "));

        // Build constraint indicators
        let mut constraint_info = String::new();
        for constraint in &flag.constraints {
            match constraint {
                FlagConstraint::RequiredIf(other) => {
                    let _ = write!(
                        &mut constraint_info,
                        " {}",
                        color::yellow(&format!("[required if --{other}]"))
                    );
                }
                FlagConstraint::ConflictsWith(others) => {
                    let conflicts = others.join(", --");
                    let _ = write!(
                        &mut constraint_info,
                        " {}",
                        color::yellow(&format!("[conflicts with --{conflicts}]"))
                    );
                }
                FlagConstraint::Requires(others) => {
                    let requires = others.join(", --");
                    let _ = write!(
                        &mut constraint_info,
                        " {}",
                        color::yellow(&format!("[requires --{requires}]"))
                    );
                }
            }
        }

        // Handle special formatting for Choice and Range types
        match &flag.value_type {
            FlagType::Choice(choices) => {
                let choices_str = choices.join("|");
                let default = flag
                    .default
                    .as_ref()
                    .map(|d| match d {
                        FlagValue::String(s) => format!(" (default \"{s}\")"),
                        _ => String::new(),
                    })
                    .unwrap_or_default();

                let flag_name_formatted = format!("{} {{{}}}", flag.name, choices_str);
                let left_part = format!(
                    "      {}--{}",
                    color::cyan(&short),
                    color::cyan(&flag_name_formatted)
                );

                let description =
                    format!("{}{}{}", flag.usage, color::dim(&default), constraint_info);
                let terminal_width = get_terminal_width();
                let left_column_width = 30;

                let formatted =
                    format_help_entry(&left_part, &description, left_column_width, terminal_width);
                println!("{formatted}");
                return;
            }
            FlagType::Range(min, max) => {
                let default = flag
                    .default
                    .as_ref()
                    .map(|d| match d {
                        FlagValue::Int(i) => format!(" (default {i})"),
                        _ => String::new(),
                    })
                    .unwrap_or_default();

                let flag_name_formatted = format!("{} int[{}-{}]", flag.name, min, max);
                let left_part = format!(
                    "      {}--{}",
                    color::cyan(&short),
                    color::cyan(&flag_name_formatted)
                );

                let description =
                    format!("{}{}{}", flag.usage, color::dim(&default), constraint_info);
                let terminal_width = get_terminal_width();
                let left_column_width = 30;

                let formatted =
                    format_help_entry(&left_part, &description, left_column_width, terminal_width);
                println!("{formatted}");
                return;
            }
            _ => {}
        }

        let flag_type = match &flag.value_type {
            FlagType::String => " string",
            FlagType::Int => " int",
            FlagType::Float => " float",
            FlagType::Bool => "",
            FlagType::StringSlice | FlagType::StringArray => " strings",
            FlagType::File => " file",
            FlagType::Directory => " dir",
            FlagType::Choice(_) | FlagType::Range(_, _) => unreachable!(),
        };

        let default = flag
            .default
            .as_ref()
            .map(|d| match d {
                FlagValue::String(s) => format!(" (default \"{s}\")"),
                FlagValue::Bool(b) => format!(" (default {b})"),
                FlagValue::Int(i) => format!(" (default {i})"),
                FlagValue::Float(f) => format!(" (default {f})"),
                FlagValue::StringSlice(v) => format!(" (default {v:?})"),
            })
            .unwrap_or_default();

        let flag_name_formatted = format!("{}{flag_type}", flag.name);
        let left_part = format!(
            "      {}--{}",
            color::cyan(&short),
            color::cyan(&flag_name_formatted)
        );

        let description = format!("{}{}{}", flag.usage, color::dim(&default), constraint_info);
        let terminal_width = get_terminal_width();
        let left_column_width = 30; // Adjust based on typical flag length

        let formatted =
            format_help_entry(&left_part, &description, left_column_width, terminal_width);
        println!("{formatted}");
    }

    /// Finds command suggestions based on similarity
    fn find_command_suggestions(&self, input: &str) -> Vec<String> {
        let candidates: Vec<String> = self.subcommands.keys().cloned().collect();
        find_suggestions(input, &candidates, self.suggestion_distance)
    }

    /// Handles shell completion requests
    ///
    /// This method is called when the shell requests completions via the
    /// environment variable (e.g., `MYAPP_COMPLETE=bash`).
    pub fn handle_completion_request(&self, args: &[String]) -> Result<Vec<String>> {
        // Detect shell type from environment variable
        let shell_type = self.detect_completion_shell();

        // args format: ["__complete", ...previous_args, current_word]
        if args.is_empty() || args[0] != "__complete" {
            return Err(Error::Completion("Invalid completion request".to_string()));
        }

        let args = &args[1..];
        if args.is_empty() {
            // Complete root level
            return Ok(self.get_completion_suggestions("", None, shell_type.as_deref()));
        }

        let current_word = args.last().unwrap_or(&String::new()).clone();
        let previous_args = &args[..args.len().saturating_sub(1)];

        // Parse through the command hierarchy
        let mut current_cmd = self;
        let mut ctx = Context::new(vec![]);
        let mut i = 0;

        while i < previous_args.len() {
            let arg = &previous_args[i];

            if arg.starts_with("--") {
                // Long flag
                let flag_name = arg.trim_start_matches("--");
                if let Some((name, _)) = flag_name.split_once('=') {
                    // Flag with value
                    ctx.set_flag(name.to_string(), String::new());
                } else if let Some(_flag) = current_cmd.find_flag(flag_name) {
                    // Flag that might need a value
                    if i + 1 < previous_args.len() && !previous_args[i + 1].starts_with('-') {
                        ctx.set_flag(flag_name.to_string(), previous_args[i + 1].clone());
                        i += 1;
                    }
                }
            } else if arg.starts_with('-') && arg.len() > 1 {
                // Short flags
                let chars = arg.chars().skip(1).collect::<Vec<_>>();
                for ch in chars {
                    if let Some(flag) = current_cmd.find_flag_by_short(ch) {
                        ctx.set_flag(flag.name.clone(), String::new());
                    }
                }
            } else {
                // Potential subcommand
                if let Some(subcmd) = current_cmd.find_subcommand(arg) {
                    current_cmd = subcmd;
                } else {
                    ctx.args_mut().push(arg.clone());
                }
            }
            i += 1;
        }

        // Now determine what to complete
        if current_word.starts_with("--") {
            // Complete long flags
            let prefix = current_word.trim_start_matches("--");
            let mut flag_completions = CompletionResult::new();

            // Collect flags with descriptions from current command and parents
            collect_all_flags_with_descriptions(current_cmd, &mut flag_completions, prefix);

            let format = CompletionFormat::from_shell_type(shell_type.as_deref());
            Ok(format.format(&flag_completions, Some(&ctx)))
        } else if current_word.starts_with('-') && current_word.len() > 1 {
            // For short flags, we don't complete (too complex)
            Ok(vec![])
        } else {
            // Check if previous arg was a flag that needs a value
            if let Some(prev) = previous_args.last() {
                if prev.starts_with("--") {
                    let flag_name = prev.trim_start_matches("--");

                    // First check if the flag itself has a completion function
                    if let Some(flag) = current_cmd.flags.get(flag_name) {
                        if let Some(ref completion_func) = flag.completion {
                            let result = completion_func(&ctx, &current_word)?;
                            let format = CompletionFormat::from_shell_type(shell_type.as_deref());
                            return Ok(format.format(&result, Some(&ctx)));
                        }
                    }

                    // Fall back to flag_completions HashMap
                    if let Some(completion_func) = current_cmd.flag_completions.get(flag_name) {
                        let result = completion_func(&ctx, &current_word)?;
                        let format = CompletionFormat::from_shell_type(shell_type.as_deref());
                        return Ok(format.format(&result, Some(&ctx)));
                    }
                } else if prev.starts_with('-') && prev.len() == 2 {
                    // Handle short flag completions
                    let short_flag = prev.chars().nth(1).unwrap();
                    if let Some(flag) = current_cmd.find_flag_by_short(short_flag) {
                        if let Some(ref completion_func) = flag.completion {
                            let result = completion_func(&ctx, &current_word)?;
                            let format = CompletionFormat::from_shell_type(shell_type.as_deref());
                            return Ok(format.format(&result, Some(&ctx)));
                        }

                        // Also check flag_completions HashMap by flag name
                        if let Some(completion_func) = current_cmd.flag_completions.get(&flag.name)
                        {
                            let result = completion_func(&ctx, &current_word)?;
                            let format = CompletionFormat::from_shell_type(shell_type.as_deref());
                            return Ok(format.format(&result, Some(&ctx)));
                        }
                    }
                }
            }

            // Complete subcommands or arguments
            Ok(current_cmd.get_completion_suggestions(
                &current_word,
                Some(&ctx),
                shell_type.as_deref(),
            ))
        }
    }

    /// Detects the shell type from the environment variable
    fn detect_completion_shell(&self) -> Option<String> {
        use std::env;

        // Look for shell-specific completion environment variables
        let env_var = format!("{}_COMPLETE", self.name.to_uppercase());
        env::var(&env_var).ok()
    }

    fn get_completion_suggestions(
        &self,
        prefix: &str,
        ctx: Option<&Context>,
        shell_type: Option<&str>,
    ) -> Vec<String> {
        let mut completion_result = CompletionResult::new();
        let mut has_suggestions = false;

        // Add subcommands with their descriptions
        for (name, cmd) in &self.subcommands {
            if name.starts_with(prefix) {
                completion_result =
                    completion_result.add_with_description(name.clone(), cmd.short.clone());
                has_suggestions = true;
            }
            // Also check aliases
            for alias in &cmd.aliases {
                if alias.starts_with(prefix) {
                    completion_result = completion_result
                        .add_with_description(alias.clone(), format!("Alias for {name}"));
                    has_suggestions = true;
                }
            }
        }

        // If we have arg completions and no subcommands match, try those
        if !has_suggestions {
            if let Some(ref completion_func) = self.arg_completions {
                let default_ctx = Context::new(vec![]);
                let ctx = ctx.unwrap_or(&default_ctx);
                if let Ok(result) = completion_func(ctx, prefix) {
                    let format = CompletionFormat::from_shell_type(shell_type);
                    return format.format(&result, Some(ctx));
                }
            }
        }

        // Format the results
        let format = CompletionFormat::from_shell_type(shell_type);
        let default_ctx = Context::new(vec![]);
        let ctx_to_use = ctx.unwrap_or(&default_ctx);
        let mut suggestions = format.format(&completion_result, Some(ctx_to_use));
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }
}

/// Builder for creating commands with a fluent API
///
/// `CommandBuilder` provides a convenient way to construct commands
/// with method chaining. This is the recommended way to create commands.
///
/// # Examples
///
/// ```rust
/// use flag_rs::{CommandBuilder, Flag, FlagType, FlagValue};
///
/// let cmd = CommandBuilder::new("serve")
///     .short("Start the web server")
///     .long("Start the web server on the specified port with the given configuration")
///     .aliases(vec!["server", "s"])
///     .flag(
///         Flag::new("port")
///             .short('p')
///             .usage("Port to listen on")
///             .value_type(FlagType::Int)
///             .default(FlagValue::Int(8080))
///     )
///     .flag(
///         Flag::new("config")
///             .short('c')
///             .usage("Configuration file path")
///             .value_type(FlagType::String)
///             .required()
///     )
///     .run(|ctx| {
///         let port = ctx.flag("port")
///             .and_then(|s| s.parse::<i64>().ok())
///             .unwrap_or(8080);
///         let config = ctx.flag("config")
///             .map(|s| s.as_str())
///             .unwrap_or("config.toml");
///
///         println!("Starting server on port {} with config {}", port, config);
///         Ok(())
///     })
///     .build();
/// ```
pub struct CommandBuilder {
    command: Command,
}

impl CommandBuilder {
    /// Creates a new command builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            command: Command::new(name),
        }
    }

    /// Adds a single alias for this command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("remove")
    ///     .alias("rm")
    ///     .alias("delete")
    ///     .build();
    /// ```
    #[must_use]
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.command.aliases.push(alias.into());
        self
    }

    /// Adds multiple aliases for this command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("remove")
    ///     .aliases(vec!["rm", "delete", "del"])
    ///     .build();
    /// ```
    #[must_use]
    pub fn aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command
            .aliases
            .extend(aliases.into_iter().map(Into::into));
        self
    }

    /// Sets the short description for this command
    ///
    /// The short description is shown in the parent command's help output.
    #[must_use]
    pub fn short(mut self, short: impl Into<String>) -> Self {
        self.command.short = short.into();
        self
    }

    /// Sets the long description for this command
    ///
    /// The long description is shown in this command's help output.
    #[must_use]
    pub fn long(mut self, long: impl Into<String>) -> Self {
        self.command.long = long.into();
        self
    }

    /// Adds an example for this command
    ///
    /// Examples are shown in the help output to demonstrate command usage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("deploy")
    ///     .short("Deploy the application")
    ///     .example("deploy --env production")
    ///     .example("deploy --env staging --dry-run")
    ///     .build();
    /// ```
    #[must_use]
    pub fn example(mut self, example: impl Into<String>) -> Self {
        self.command.examples.push(example.into());
        self
    }

    /// Sets the group ID for this command
    ///
    /// Commands with the same group ID will be displayed together in help output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let app = CommandBuilder::new("kubectl")
    ///     .subcommand(
    ///         CommandBuilder::new("get")
    ///             .short("Display resources")
    ///             .group_id("Basic Commands")
    ///             .build()
    ///     )
    ///     .subcommand(
    ///         CommandBuilder::new("create")
    ///             .short("Create resources")
    ///             .group_id("Basic Commands")
    ///             .build()
    ///     )
    ///     .subcommand(
    ///         CommandBuilder::new("config")
    ///             .short("Modify kubeconfig files")
    ///             .group_id("Settings Commands")
    ///             .build()
    ///     )
    ///     .build();
    /// ```
    #[must_use]
    pub fn group_id(mut self, group_id: impl Into<String>) -> Self {
        self.command.group_id = Some(group_id.into());
        self
    }

    /// Adds a subcommand to this command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let app = CommandBuilder::new("myapp")
    ///     .subcommand(
    ///         CommandBuilder::new("init")
    ///             .short("Initialize a new project")
    ///             .build()
    ///     )
    ///     .subcommand(
    ///         CommandBuilder::new("build")
    ///             .short("Build the project")
    ///             .build()
    ///     )
    ///     .build();
    /// ```
    #[must_use]
    pub fn subcommand(mut self, cmd: Command) -> Self {
        self.command.add_command(cmd);
        self
    }

    /// Adds multiple subcommands to this command at once
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("git")
    ///     .subcommands(vec![
    ///         CommandBuilder::new("add")
    ///             .short("Add file contents to the index")
    ///             .build(),
    ///         CommandBuilder::new("commit")
    ///             .short("Record changes to the repository")
    ///             .build(),
    ///         CommandBuilder::new("push")
    ///             .short("Update remote refs along with associated objects")
    ///             .build(),
    ///     ])
    ///     .build();
    /// ```
    #[must_use]
    pub fn subcommands(mut self, cmds: Vec<Command>) -> Self {
        for cmd in cmds {
            self.command.add_command(cmd);
        }
        self
    }

    /// Adds a flag to this command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{CommandBuilder, Flag, FlagType};
    ///
    /// let cmd = CommandBuilder::new("deploy")
    ///     .flag(
    ///         Flag::new("force")
    ///             .short('f')
    ///             .usage("Force deployment without confirmation")
    ///             .value_type(FlagType::Bool)
    ///     )
    ///     .build();
    /// ```
    #[must_use]
    pub fn flag(mut self, flag: Flag) -> Self {
        self.command.flags.insert(flag.name.clone(), flag);
        self
    }

    /// Adds multiple flags to this command at once
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{CommandBuilder, Flag};
    ///
    /// let cmd = CommandBuilder::new("server")
    ///     .flags(vec![
    ///         Flag::bool("verbose").short('v').usage("Enable verbose output"),
    ///         Flag::bool("quiet").short('q').usage("Suppress output"),
    ///         Flag::int("port").short('p').usage("Port to listen on").default_int(8080),
    ///     ])
    ///     .build();
    /// ```
    #[must_use]
    pub fn flags(mut self, flags: Vec<Flag>) -> Self {
        for flag in flags {
            self.command.flags.insert(flag.name.clone(), flag);
        }
        self
    }

    /// Sets the function to run when this command is executed
    ///
    /// The run function receives a mutable reference to the [`Context`]
    /// which provides access to parsed flags and arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("greet")
    ///     .run(|ctx| {
    ///         let name = ctx.args().first()
    ///             .map(|s| s.as_str())
    ///             .unwrap_or("World");
    ///         println!("Hello, {}!", name);
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn run<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    {
        self.command.run = Some(Box::new(f));
        self
    }

    /// Sets the argument validator for this command
    ///
    /// The validator will be called before the run function to ensure
    /// arguments meet the specified constraints.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{CommandBuilder, ArgValidator};
    ///
    /// let cmd = CommandBuilder::new("delete")
    ///     .args(ArgValidator::MinimumArgs(1))
    ///     .run(|ctx| {
    ///         for file in ctx.args() {
    ///             println!("Deleting: {}", file);
    ///         }
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn args(mut self, validator: ArgValidator) -> Self {
        self.command.arg_validator = Some(validator);
        self
    }

    /// Sets the persistent pre-run hook for this command
    ///
    /// This hook runs before the command and all its subcommands.
    /// It's inherited by all subcommands and runs in parent-to-child order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("app")
    ///     .persistent_pre_run(|ctx| {
    ///         println!("Setting up logging...");
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn persistent_pre_run<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    {
        self.command.persistent_pre_run = Some(Box::new(f));
        self
    }

    /// Sets the pre-run hook for this command
    ///
    /// This hook runs only for this specific command, after any persistent
    /// pre-run hooks but before the main run function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("deploy")
    ///     .pre_run(|ctx| {
    ///         println!("Validating deployment configuration...");
    ///         Ok(())
    ///     })
    ///     .run(|ctx| {
    ///         println!("Deploying application...");
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn pre_run<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    {
        self.command.pre_run = Some(Box::new(f));
        self
    }

    /// Sets the post-run hook for this command
    ///
    /// This hook runs only for this specific command, after the main run
    /// function but before any persistent post-run hooks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("test")
    ///     .run(|ctx| {
    ///         println!("Running tests...");
    ///         Ok(())
    ///     })
    ///     .post_run(|ctx| {
    ///         println!("Generating test report...");
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn post_run<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    {
        self.command.post_run = Some(Box::new(f));
        self
    }

    /// Sets the persistent post-run hook for this command
    ///
    /// This hook runs after the command and all its subcommands.
    /// It's inherited by all subcommands and runs in child-to-parent order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("app")
    ///     .persistent_post_run(|ctx| {
    ///         println!("Cleaning up resources...");
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn persistent_post_run<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    {
        self.command.persistent_post_run = Some(Box::new(f));
        self
    }

    /// Sets the argument completion function
    ///
    /// This function is called when the user presses TAB to complete arguments.
    /// It enables dynamic completions based on runtime state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{CommandBuilder, CompletionResult};
    ///
    /// let cmd = CommandBuilder::new("edit")
    ///     .arg_completion(|ctx, prefix| {
    ///         // In a real app, list files from the filesystem
    ///         let files = vec!["main.rs", "lib.rs", "Cargo.toml"];
    ///         Ok(CompletionResult::new().extend(
    ///             files.into_iter()
    ///                 .filter(|f| f.starts_with(prefix))
    ///                 .map(String::from)
    ///         ))
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn arg_completion<F>(mut self, f: F) -> Self
    where
        F: Fn(&Context, &str) -> Result<CompletionResult> + Send + Sync + 'static,
    {
        self.command.set_arg_completion(f);
        self
    }

    /// Sets the completion function for a specific flag
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType};
    ///
    /// let cmd = CommandBuilder::new("connect")
    ///     .flag(
    ///         Flag::new("server")
    ///             .usage("Server to connect to")
    ///             .value_type(FlagType::String)
    ///     )
    ///     .flag_completion("server", |ctx, prefix| {
    ///         // In a real app, discover available servers
    ///         let servers = vec!["prod-1", "prod-2", "staging", "dev"];
    ///         Ok(CompletionResult::new().extend(
    ///             servers.into_iter()
    ///                 .filter(|s| s.starts_with(prefix))
    ///                 .map(String::from)
    ///         ))
    ///     })
    ///     .build();
    /// ```
    #[must_use]
    pub fn flag_completion<F>(mut self, flag_name: impl Into<String>, f: F) -> Self
    where
        F: Fn(&Context, &str) -> Result<CompletionResult> + Send + Sync + 'static,
    {
        self.command.set_flag_completion(flag_name, f);
        self
    }

    /// Enables or disables command suggestions
    ///
    /// When enabled, the framework will suggest similar commands when
    /// a user types an unknown command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("myapp")
    ///     .suggestions(true)  // Enable suggestions (default)
    ///     .build();
    /// ```
    #[must_use]
    pub fn suggestions(mut self, enabled: bool) -> Self {
        self.command.suggestions_enabled = enabled;
        self
    }

    /// Sets the maximum Levenshtein distance for suggestions
    ///
    /// Commands within this distance will be suggested as alternatives.
    /// Default is 2.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use flag_rs::CommandBuilder;
    ///
    /// let cmd = CommandBuilder::new("myapp")
    ///     .suggestion_distance(3)  // Allow more distant suggestions
    ///     .build();
    /// ```
    #[must_use]
    pub fn suggestion_distance(mut self, distance: usize) -> Self {
        self.command.suggestion_distance = distance;
        self
    }

    /// Builds and returns the completed [`Command`]
    #[must_use]
    pub fn build(self) -> Command {
        self.command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flag::FlagType;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_simple_command_execution() {
        let executed = Arc::new(Mutex::new(false));
        let executed_clone = executed.clone();

        let cmd = CommandBuilder::new("test")
            .run(move |_ctx| {
                *executed_clone.lock().unwrap() = true;
                Ok(())
            })
            .build();

        cmd.execute(vec![]).unwrap();
        assert!(*executed.lock().unwrap());
    }

    #[test]
    fn test_command_with_args() {
        let received_args = Arc::new(Mutex::new(Vec::new()));
        let args_clone = received_args.clone();

        let cmd = CommandBuilder::new("test")
            .run(move |ctx| {
                *args_clone.lock().unwrap() = ctx.args().to_vec();
                Ok(())
            })
            .build();

        cmd.execute(vec!["arg1".to_string(), "arg2".to_string()])
            .unwrap();
        assert_eq!(*received_args.lock().unwrap(), vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_subcommand_execution() {
        let main_executed = Arc::new(Mutex::new(false));
        let sub_executed = Arc::new(Mutex::new(false));
        let sub_clone = sub_executed.clone();

        let subcmd = CommandBuilder::new("sub")
            .run(move |_ctx| {
                *sub_clone.lock().unwrap() = true;
                Ok(())
            })
            .build();

        let main_clone = main_executed.clone();
        let cmd = CommandBuilder::new("main")
            .run(move |_ctx| {
                *main_clone.lock().unwrap() = true;
                Ok(())
            })
            .subcommand(subcmd)
            .build();

        // Execute subcommand
        cmd.execute(vec!["sub".to_string()]).unwrap();
        assert!(*sub_executed.lock().unwrap());
        assert!(!*main_executed.lock().unwrap());
    }

    #[test]
    fn test_flag_parsing() {
        let cmd = CommandBuilder::new("test")
            .flag(Flag::new("verbose").short('v').value_type(FlagType::Bool))
            .flag(Flag::new("output").short('o').value_type(FlagType::String))
            .flag(Flag::new("count").value_type(FlagType::Int))
            .run(|ctx| {
                assert_eq!(ctx.flag("verbose"), Some(&"true".to_string()));
                assert_eq!(ctx.flag("output"), Some(&"file.txt".to_string()));
                assert_eq!(ctx.flag("count"), Some(&"42".to_string()));
                assert_eq!(ctx.args(), &["remaining"]);
                Ok(())
            })
            .build();

        cmd.execute(vec![
            "-v".to_string(),
            "--output".to_string(),
            "file.txt".to_string(),
            "--count=42".to_string(),
            "remaining".to_string(),
        ])
        .unwrap();
    }

    #[test]
    fn test_flag_inheritance() {
        let sub_executed = Arc::new(Mutex::new(false));
        let sub_clone = sub_executed.clone();

        let subcmd = CommandBuilder::new("sub")
            .run(move |ctx| {
                assert_eq!(ctx.flag("global"), Some(&"value".to_string()));
                *sub_clone.lock().unwrap() = true;
                Ok(())
            })
            .build();

        let cmd = CommandBuilder::new("main")
            .flag(Flag::new("global").value_type(FlagType::String))
            .subcommand(subcmd)
            .build();

        cmd.execute(vec![
            "--global".to_string(),
            "value".to_string(),
            "sub".to_string(),
        ])
        .unwrap();

        assert!(*sub_executed.lock().unwrap());
    }

    #[test]
    fn test_command_aliases() {
        let executed = Arc::new(Mutex::new(String::new()));
        let exec_clone = executed.clone();

        let subcmd = CommandBuilder::new("subcommand")
            .aliases(vec!["sub", "s"])
            .run(move |_ctx| {
                *exec_clone.lock().unwrap() = "subcommand".to_string();
                Ok(())
            })
            .build();

        let cmd = CommandBuilder::new("main").subcommand(subcmd).build();

        // Test main name
        cmd.execute(vec!["subcommand".to_string()]).unwrap();
        assert_eq!(*executed.lock().unwrap(), "subcommand");

        // Test alias
        cmd.execute(vec!["sub".to_string()]).unwrap();
        assert_eq!(*executed.lock().unwrap(), "subcommand");

        // Test short alias
        cmd.execute(vec!["s".to_string()]).unwrap();
        assert_eq!(*executed.lock().unwrap(), "subcommand");
    }

    #[test]
    fn test_error_cases() {
        let cmd = CommandBuilder::new("main").build();

        // No subcommand when required
        let result = cmd.execute(vec![]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SubcommandRequired(_)));

        // Unknown subcommand
        let result = cmd.execute(vec!["unknown".to_string()]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::CommandNotFound { .. }));

        // Unknown flag (now treated as argument, so it becomes unknown command)
        let result = cmd.execute(vec!["--unknown".to_string()]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::CommandNotFound { .. }));
    }

    #[test]
    fn test_completion() {
        let cmd = CommandBuilder::new("test")
            .arg_completion(|_ctx, prefix| {
                Ok(CompletionResult::new().extend(
                    vec!["file1.txt", "file2.txt", "folder/"]
                        .into_iter()
                        .filter(|f| f.starts_with(prefix))
                        .map(String::from),
                ))
            })
            .flag_completion("type", |_ctx, prefix| {
                Ok(CompletionResult::new().extend(
                    vec!["json", "yaml", "xml"]
                        .into_iter()
                        .filter(|t| t.starts_with(prefix))
                        .map(String::from),
                ))
            })
            .build();

        let ctx = Context::new(vec![]);

        // Test arg completion
        let result = cmd.get_completions(&ctx, "fi", None).unwrap();
        assert_eq!(result.values, vec!["file1.txt", "file2.txt"]);

        // Test flag completion
        let result = cmd.get_completions(&ctx, "j", Some("type")).unwrap();
        assert_eq!(result.values, vec!["json"]);
    }

    #[test]
    fn test_flag_with_equals() {
        let cmd = CommandBuilder::new("test")
            .flag(Flag::new("output").value_type(FlagType::String))
            .run(|ctx| {
                assert_eq!(
                    ctx.flag("output"),
                    Some(&"/path/with=equals.txt".to_string())
                );
                Ok(())
            })
            .build();

        cmd.execute(vec!["--output=/path/with=equals.txt".to_string()])
            .unwrap();
    }

    #[test]
    fn test_help_flag() {
        let cmd = CommandBuilder::new("test")
            .short("Test command")
            .long("This is a test command")
            .flag(
                Flag::new("verbose")
                    .short('v')
                    .usage("Enable verbose output"),
            )
            .build();

        // Test --help
        let result = cmd.execute(vec!["--help".to_string()]);
        assert!(result.is_ok());

        // Test -h
        let result = cmd.execute(vec!["-h".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_subcommand_help() {
        let subcmd = CommandBuilder::new("sub")
            .short("Subcommand")
            .flag(Flag::new("subflag").usage("A flag for the subcommand"))
            .build();

        let cmd = CommandBuilder::new("main")
            .flag(Flag::new("global").usage("A global flag"))
            .subcommand(subcmd)
            .build();

        // Test help on subcommand
        let result = cmd.execute(vec!["sub".to_string(), "--help".to_string()]);
        assert!(result.is_ok());
    }
}
