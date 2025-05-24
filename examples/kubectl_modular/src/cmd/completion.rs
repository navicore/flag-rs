use flag_rs::{Command, CommandBuilder, Shell};

pub fn register(parent: &mut Command) {
    let cmd = CommandBuilder::new("completion")
        .short("Generate shell completion scripts")
        .long("Generate shell completion scripts for kubectl.\n\nTo load completions:\n\nBash:\n  $ source <(kubectl completion bash)\n\n  # To load completions for each session, execute once:\n  # Linux:\n  $ kubectl completion bash > /etc/bash_completion.d/kubectl\n  # macOS:\n  $ kubectl completion bash > /usr/local/etc/bash_completion.d/kubectl\n\nZsh:\n  $ source <(kubectl completion zsh)\n\n  # To load completions for each session, execute once:\n  $ kubectl completion zsh > \"${fpath[1]}/_kubectl\"\n\nFish:\n  $ kubectl completion fish | source\n\n  # To load completions for each session, execute once:\n  $ kubectl completion fish > ~/.config/fish/completions/kubectl.fish")
        .run(|ctx| {
            let shell_name = ctx.args().first()
                .ok_or_else(|| flag_rs::Error::ArgumentParsing("shell name required (bash, zsh, or fish)".to_string()))?;

            let shell = match shell_name.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                _ => return Err(flag_rs::Error::ArgumentParsing(format!("unsupported shell: {}", shell_name))),
            };

            // Get the root command
            let root = get_root_command();
            println!("{}", root.generate_completion(shell));

            Ok(())
        })
        .build();

    parent.add_command(cmd);
}

// This is a bit of a hack - in a real implementation, you'd pass the root command
// through the registration chain or store it in a context
fn get_root_command() -> Command {
    // Rebuild the command structure for completion generation
    let mut app = CommandBuilder::new("kubectl")
        .short("Kubernetes command-line tool")
        .long("kubectl controls the Kubernetes cluster manager")
        .build();

    // We only need the structure, not the implementations
    app.add_command(CommandBuilder::new("get").build());
    app.add_command(CommandBuilder::new("describe").build());
    app.add_command(CommandBuilder::new("delete").build());
    app.add_command(CommandBuilder::new("apply").build());
    app.add_command(CommandBuilder::new("completion").build());

    app
}
