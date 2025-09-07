//! Shell completion script generation
//!
//! This module provides functionality to generate shell completion scripts
//! for Bash, Zsh, and Fish shells. The generated scripts integrate with the
//! dynamic completion system to provide TAB completions at runtime.

use crate::command::Command;
use std::fmt::Write;

/// Safe writeln macro that handles the rare case where writing to String fails
/// Writing to String should virtually never fail except in extreme memory conditions
macro_rules! safe_writeln {
    ($dst:expr) => {
        if writeln!($dst).is_err() {
            eprintln!("Warning: Failed to write to completion script buffer");
        }
    };
    ($dst:expr, $($arg:tt)*) => {
        if writeln!($dst, $($arg)*).is_err() {
            eprintln!("Warning: Failed to write to completion script buffer");
        }
    };
}

/// Supported shell types for completion generation
///
/// This enum represents the shells for which we can generate completion scripts.
///
/// # Examples
///
/// ```
/// use flag_rs::shell::Shell;
/// use flag_rs::Command;
///
/// let cmd = Command::new("myapp");
///
/// // Generate Bash completion script
/// let bash_script = cmd.generate_completion(Shell::Bash);
///
/// // Generate Zsh completion script
/// let zsh_script = cmd.generate_completion(Shell::Zsh);
///
/// // Generate Fish completion script
/// let fish_script = cmd.generate_completion(Shell::Fish);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Shell {
    /// Bash shell (most common on Linux)
    Bash,
    /// Zsh shell (default on macOS)
    Zsh,
    /// Fish shell (modern alternative shell)
    Fish,
}

impl Command {
    /// Generates a completion script for the specified shell
    ///
    /// The generated script should be saved to the appropriate location
    /// for your shell to load it automatically.
    ///
    /// # Arguments
    ///
    /// * `shell` - The shell to generate completions for
    ///
    /// # Returns
    ///
    /// A string containing the shell completion script
    ///
    /// # Shell-specific installation
    ///
    /// ## Bash
    /// Save to `/etc/bash_completion.d/myapp` or source from `.bashrc`:
    /// ```bash
    /// myapp completion bash > ~/.myapp-completion.bash
    /// echo "source ~/.myapp-completion.bash" >> ~/.bashrc
    /// ```
    ///
    /// ## Zsh
    /// Save to a directory in your `$fpath`:
    /// ```bash
    /// myapp completion zsh > ~/.zsh/completions/_myapp
    /// ```
    ///
    /// ## Fish
    /// Save to Fish's completion directory:
    /// ```bash
    /// myapp completion fish > ~/.config/fish/completions/myapp.fish
    /// ```
    pub fn generate_completion(&self, shell: Shell) -> String {
        match shell {
            Shell::Bash => self.generate_bash_completion(),
            Shell::Zsh => self.generate_zsh_completion(),
            Shell::Fish => self.generate_fish_completion(),
        }
    }

    fn generate_bash_completion(&self) -> String {
        let mut script = String::new();

        safe_writeln!(&mut script, "# Bash completion for {}", self.name());
        safe_writeln!(&mut script, "_{}_complete() {{", self.name());
        safe_writeln!(&mut script, "    local cur prev words cword");
        safe_writeln!(
            &mut script,
            "    _get_comp_words_by_ref -n : cur prev words cword"
        );
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "    # Call our binary with special completion env var"
        );
        safe_writeln!(&mut script, "    local IFS=$'\\n'");
        safe_writeln!(&mut script, "    local response");
        safe_writeln!(
            &mut script,
            "    response=$({}_COMPLETE=bash \"${{words[0]}}\" __complete \"${{words[@]:1:$((cword-1))}}\" \"$cur\" 2>/dev/null)",
            self.name().to_uppercase()
        );
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    if [[ -n \"$response\" ]]; then");
        safe_writeln!(
            &mut script,
            "        # Use printf to handle each line separately"
        );
        safe_writeln!(&mut script, "        local lines=()");
        safe_writeln!(&mut script, "        local help_messages=()");
        safe_writeln!(&mut script, "        while IFS= read -r line; do");
        safe_writeln!(
            &mut script,
            "            if [[ \"$line\" == _activehelp_* ]]; then"
        );
        safe_writeln!(&mut script, "                # Extract help message");
        safe_writeln!(
            &mut script,
            "                help_messages+=(\"${{line#_activehelp_ }}\")"
        );
        safe_writeln!(&mut script, "            else");
        safe_writeln!(&mut script, "                lines+=(\"$line\")");
        safe_writeln!(&mut script, "            fi");
        safe_writeln!(&mut script, "        done <<< \"$response\"");
        safe_writeln!(&mut script, "        COMPREPLY=( \"${{lines[@]}}\" )");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "        # Display help messages if any");
        safe_writeln!(
            &mut script,
            "        if [[ ${{#help_messages[@]}} -gt 0 ]]; then"
        );
        safe_writeln!(&mut script, "            printf '\\n'");
        safe_writeln!(
            &mut script,
            "            for msg in \"${{help_messages[@]}}\"; do"
        );
        safe_writeln!(&mut script, "                printf '%s\\n' \"$msg\"");
        safe_writeln!(&mut script, "            done");
        safe_writeln!(&mut script, "            printf '\\n'");
        safe_writeln!(&mut script, "        fi");
        safe_writeln!(&mut script, "    fi");
        safe_writeln!(&mut script, "}}");
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "complete -F _{}_complete {}",
            self.name(),
            self.name()
        );

        script
    }

    fn generate_zsh_completion(&self) -> String {
        let mut script = String::new();

        safe_writeln!(&mut script, "#compdef -P {}", self.name());
        safe_writeln!(&mut script, "# Zsh completion for {}", self.name());
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "_{}_complete() {{", self.name());
        safe_writeln!(&mut script, "    local -a completions");
        safe_writeln!(&mut script, "    local IFS=$'\\n'");
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "    # Get the actual command from the command line"
        );
        safe_writeln!(&mut script, "    local cmd=\"${{words[1]}}\"");
        safe_writeln!(
            &mut script,
            "    if [[ \"$cmd\" != /* ]] && ! command -v \"$cmd\" &>/dev/null; then"
        );
        safe_writeln!(
            &mut script,
            "        # If not found in PATH, try relative path"
        );
        safe_writeln!(&mut script, "        if [[ -x \"$cmd\" ]]; then");
        safe_writeln!(&mut script, "            cmd=\"./$cmd\"");
        safe_writeln!(&mut script, "        fi");
        safe_writeln!(&mut script, "    fi");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    # Build completion arguments");
        safe_writeln!(&mut script, "    local -a comp_line");
        safe_writeln!(&mut script, "    comp_line=(\"__complete\")");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    # Add all words except the command name");
        safe_writeln!(&mut script, "    local i");
        safe_writeln!(&mut script, "    for (( i = 2; i < CURRENT; i++ )); do");
        safe_writeln!(&mut script, "        comp_line+=(\"${{words[$i]}}\")");
        safe_writeln!(&mut script, "    done");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    # Add the current word being completed");
        safe_writeln!(&mut script, "    comp_line+=(\"${{words[CURRENT]}}\")");
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "    # Call the command with completion environment variable"
        );
        safe_writeln!(&mut script, "    local response");
        safe_writeln!(
            &mut script,
            "    response=$({}_COMPLETE=zsh \"$cmd\" \"${{comp_line[@]}}\" 2>/dev/null)",
            self.name().to_uppercase()
        );
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    if [[ -n \"$response\" ]]; then");
        safe_writeln!(&mut script, "        local -a values");
        safe_writeln!(&mut script, "        local -a descriptions");
        safe_writeln!(&mut script, "        local -a help_messages");
        safe_writeln!(&mut script, "        local line");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "        # Parse response lines");
        safe_writeln!(&mut script, "        while IFS= read -r line; do");
        safe_writeln!(
            &mut script,
            "            if [[ \"$line\" == _activehelp_::* ]]; then"
        );
        safe_writeln!(&mut script, "                # ActiveHelp message");
        safe_writeln!(
            &mut script,
            "                help_messages+=(\"${{line#_activehelp_::}}\")"
        );
        safe_writeln!(&mut script, "            elif [[ \"$line\" == *:* ]]; then");
        safe_writeln!(&mut script, "                # Line has description");
        safe_writeln!(&mut script, "                values+=(\"${{line%%:*}}\")");
        safe_writeln!(
            &mut script,
            "                descriptions+=(\"${{line#*:}}\")"
        );
        safe_writeln!(&mut script, "            else");
        safe_writeln!(&mut script, "                # No description");
        safe_writeln!(&mut script, "                values+=(\"$line\")");
        safe_writeln!(&mut script, "                descriptions+=(\"\")");
        safe_writeln!(&mut script, "            fi");
        safe_writeln!(&mut script, "        done <<< \"$response\"");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "        # Display ActiveHelp messages if any");
        safe_writeln!(
            &mut script,
            "        if [[ ${{#help_messages[@]}} -gt 0 ]]; then"
        );
        safe_writeln!(&mut script, "            local formatted_help=()");
        safe_writeln!(
            &mut script,
            "            for msg in \"${{help_messages[@]}}\"; do"
        );
        safe_writeln!(
            &mut script,
            "                formatted_help+=(\"-- $msg --\")"
        );
        safe_writeln!(&mut script, "            done");
        safe_writeln!(
            &mut script,
            "            compadd -x \"${{(j: :)formatted_help}}\""
        );
        safe_writeln!(&mut script, "        fi");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "        # Add completions with descriptions");
        safe_writeln!(
            &mut script,
            "        if [[ ${{#descriptions[@]}} -gt 0 ]] && [[ -n \"${{descriptions[*]// }}\" ]]; then"
        );
        safe_writeln!(
            &mut script,
            "            compadd -Q -d descriptions -a values"
        );
        safe_writeln!(&mut script, "        else");
        safe_writeln!(&mut script, "            compadd -Q -a values");
        safe_writeln!(&mut script, "        fi");
        safe_writeln!(&mut script, "    fi");
        safe_writeln!(&mut script, "}}");
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "compdef _{}_complete {}",
            self.name(),
            self.name()
        );

        script
    }

    fn generate_fish_completion(&self) -> String {
        let mut script = String::new();

        safe_writeln!(&mut script, "# Fish completion for {}", self.name());
        safe_writeln!(&mut script, "function __{}_complete", self.name());
        safe_writeln!(&mut script, "    set -l cmd (commandline -opc)");
        safe_writeln!(&mut script, "    set -l cursor (commandline -C)");
        safe_writeln!(&mut script, "    set -l current (commandline -ct)");
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "    # Call our binary with special completion env var"
        );
        safe_writeln!(
            &mut script,
            "    set -l response (env {}_COMPLETE=fish $cmd[1] __complete $cmd[2..-1] $current 2>/dev/null)",
            self.name().to_uppercase()
        );
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    # Process response and handle ActiveHelp");
        safe_writeln!(&mut script, "    set -l help_messages");
        safe_writeln!(&mut script, "    for line in $response");
        safe_writeln!(
            &mut script,
            "        if string match -q '_activehelp_*' -- $line"
        );
        safe_writeln!(&mut script, "            # Extract help message");
        safe_writeln!(
            &mut script,
            "            set -a help_messages (string replace '_activehelp_\t' '' -- $line)"
        );
        safe_writeln!(&mut script, "        else");
        safe_writeln!(&mut script, "            echo $line");
        safe_writeln!(&mut script, "        end");
        safe_writeln!(&mut script, "    end");
        safe_writeln!(&mut script);
        safe_writeln!(&mut script, "    # Display help messages if any");
        safe_writeln!(&mut script, "    if test (count $help_messages) -gt 0");
        safe_writeln!(&mut script, "        for msg in $help_messages");
        safe_writeln!(&mut script, "            echo \"» $msg\" >&2");
        safe_writeln!(&mut script, "        end");
        safe_writeln!(&mut script, "    end");
        safe_writeln!(&mut script, "end");
        safe_writeln!(&mut script);
        safe_writeln!(
            &mut script,
            "complete -c {} -f -a '(__{}_complete)'",
            self.name(),
            self.name()
        );

        script
    }
}
