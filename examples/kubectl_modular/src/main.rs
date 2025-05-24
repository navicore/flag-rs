mod cmd;

use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType, FlagValue};
use std::env;

fn main() {
    // Create root command with global flags
    let mut app = CommandBuilder::new("kubectl")
        .short("Kubernetes command-line tool")
        .long("kubectl controls the Kubernetes cluster manager")
        .flag(
            Flag::new("namespace")
                .short('n')
                .usage("If present, the namespace scope for this CLI request")
                .value_type(FlagType::String)
                .default(FlagValue::String("default".to_string())),
        )
        .flag(
            Flag::new("kubeconfig")
                .usage("Path to the kubeconfig file to use for CLI requests")
                .value_type(FlagType::String),
        )
        .flag(
            Flag::new("context")
                .usage("The name of the kubeconfig context to use")
                .value_type(FlagType::String),
        )
        .flag_completion("namespace", |_ctx, prefix| {
            // In a real implementation, this would query the API server
            let namespaces = vec![
                "default",
                "kube-system",
                "kube-public",
                "kube-node-lease",
                "development",
                "staging",
                "production",
            ];

            Ok(CompletionResult::new().extend(
                namespaces
                    .into_iter()
                    .filter(|ns| ns.starts_with(prefix))
                    .map(String::from)
                    .collect::<Vec<_>>(),
            ))
        })
        .build();

    // Register all subcommands
    cmd::register_commands(&mut app);

    // Execute
    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
