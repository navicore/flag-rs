//! Shell completion script generation
//!
//! This module provides functionality to generate shell completion scripts
//! for Bash, Zsh, and Fish shells. The generated scripts integrate with the
//! dynamic completion system to provide TAB completions at runtime.

use crate::command::Command;
use std::fmt::Write;

/// Supported shell types for completion generation
///
/// This enum represents the shells for which we can generate completion scripts.
///
/// # Examples
///
/// ```
/// use flag::shell::Shell;
/// use flag::Command;
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

        writeln!(&mut script, "# Bash completion for {}", self.name()).unwrap();
        writeln!(&mut script, "_{}_complete() {{", self.name()).unwrap();
        writeln!(&mut script, "    local cur prev words cword").unwrap();
        writeln!(
            &mut script,
            "    _get_comp_words_by_ref -n : cur prev words cword"
        )
        .unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "    # Call our binary with special completion env var"
        )
        .unwrap();
        writeln!(&mut script, "    local IFS=$'\\n'").unwrap();
        writeln!(&mut script, "    local response").unwrap();
        writeln!(&mut script, "    response=$({}_COMPLETE=bash \"${{words[0]}}\" __complete \"${{words[@]:1:$((cword-1))}}\" \"$cur\" 2>/dev/null)", self.name().to_uppercase()).unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "    if [[ -n \"$response\" ]]; then").unwrap();
        writeln!(
            &mut script,
            "        COMPREPLY=( $(compgen -W \"$response\" -- \"$cur\") )"
        )
        .unwrap();
        writeln!(&mut script, "    fi").unwrap();
        writeln!(&mut script, "}}").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "complete -F _{}_complete {}",
            self.name(),
            self.name()
        )
        .unwrap();

        script
    }

    fn generate_zsh_completion(&self) -> String {
        let mut script = String::new();

        writeln!(&mut script, "#compdef -P {}", self.name()).unwrap();
        writeln!(&mut script, "# Zsh completion for {}", self.name()).unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "_{}_complete() {{", self.name()).unwrap();
        writeln!(&mut script, "    local -a completions").unwrap();
        writeln!(&mut script, "    local IFS=$'\\n'").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "    # Get the actual command from the command line"
        )
        .unwrap();
        writeln!(&mut script, "    local cmd=\"${{words[1]}}\"").unwrap();
        writeln!(
            &mut script,
            "    if [[ \"$cmd\" != /* ]] && ! command -v \"$cmd\" &>/dev/null; then"
        )
        .unwrap();
        writeln!(
            &mut script,
            "        # If not found in PATH, try relative path"
        )
        .unwrap();
        writeln!(&mut script, "        if [[ -x \"$cmd\" ]]; then").unwrap();
        writeln!(&mut script, "            cmd=\"./$cmd\"").unwrap();
        writeln!(&mut script, "        fi").unwrap();
        writeln!(&mut script, "    fi").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "    # Build completion arguments").unwrap();
        writeln!(&mut script, "    local -a comp_line").unwrap();
        writeln!(&mut script, "    comp_line=(\"__complete\")").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "    # Add all words except the command name").unwrap();
        writeln!(&mut script, "    local i").unwrap();
        writeln!(&mut script, "    for (( i = 2; i < CURRENT; i++ )); do").unwrap();
        writeln!(&mut script, "        comp_line+=(\"${{words[$i]}}\")").unwrap();
        writeln!(&mut script, "    done").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "    # Add the current word being completed").unwrap();
        writeln!(&mut script, "    comp_line+=(\"${{words[CURRENT]}}\")").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "    # Call the command with completion environment variable"
        )
        .unwrap();
        writeln!(&mut script, "    local response").unwrap();
        writeln!(
            &mut script,
            "    response=$({}_COMPLETE=zsh \"$cmd\" \"${{comp_line[@]}}\" 2>/dev/null)",
            self.name().to_uppercase()
        )
        .unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "    if [[ -n \"$response\" ]]; then").unwrap();
        writeln!(&mut script, "        completions=(${{(f)response}})").unwrap();
        writeln!(&mut script, "        compadd -a completions").unwrap();
        writeln!(&mut script, "    fi").unwrap();
        writeln!(&mut script, "}}").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "compdef _{}_complete {}",
            self.name(),
            self.name()
        )
        .unwrap();

        script
    }

    fn generate_fish_completion(&self) -> String {
        let mut script = String::new();

        writeln!(&mut script, "# Fish completion for {}", self.name()).unwrap();
        writeln!(&mut script, "function __{}_complete", self.name()).unwrap();
        writeln!(&mut script, "    set -l cmd (commandline -opc)").unwrap();
        writeln!(&mut script, "    set -l cursor (commandline -C)").unwrap();
        writeln!(&mut script, "    set -l current (commandline -ct)").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "    # Call our binary with special completion env var"
        )
        .unwrap();
        writeln!(&mut script, "    set -l response (env {}_COMPLETE=fish $cmd[1] __complete $cmd[2..-1] $current 2>/dev/null)", self.name().to_uppercase()).unwrap();
        writeln!(&mut script).unwrap();
        writeln!(&mut script, "    for line in $response").unwrap();
        writeln!(&mut script, "        echo $line").unwrap();
        writeln!(&mut script, "    end").unwrap();
        writeln!(&mut script, "end").unwrap();
        writeln!(&mut script).unwrap();
        writeln!(
            &mut script,
            "complete -c {} -f -a '(__{}_complete)'",
            self.name(),
            self.name()
        )
        .unwrap();

        script
    }
}
