use flag_rs::{Command, CommandBuilder, CompletionResult, Flag, FlagType, FlagValue, Shell};
use std::env;

fn main() {
    let app = build_kubectl();

    let args: Vec<String> = env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn build_kubectl() -> Command {
    CommandBuilder::new("kubectl")
        .short("Kubernetes command-line tool")
        .long("kubectl controls the Kubernetes cluster manager")
        .subcommand(build_get_command())
        .subcommand(build_describe_command())
        .subcommand(build_delete_command())
        .subcommand(build_completion_command())
        .flag(
            Flag::new("namespace")
                .short('n')
                .usage("Kubernetes namespace")
                .value_type(FlagType::String)
                .default(FlagValue::String("default".to_string())),
        )
        .flag_completion("namespace", |_ctx, prefix| {
            // In a real kubectl, this would query the API server
            let namespaces = get_namespaces();
            Ok(CompletionResult::new()
                .extend(namespaces.into_iter().filter(|ns| ns.starts_with(prefix))))
        })
        .build()
}

fn build_get_command() -> Command {
    CommandBuilder::new("get")
        .short("Display one or many resources")
        .long("Display one or many resources. Prints a table of the most important information about the specified resources.")
        .subcommand(build_get_pods())
        .subcommand(build_get_services())
        .subcommand(build_get_deployments())
        .build()
}

fn build_get_pods() -> Command {
    CommandBuilder::new("pods")
        .aliases(vec!["po", "pod"])
        .short("List pods")
        .arg_completion(|ctx, prefix| {
            // This is the key feature - dynamic completion based on runtime state!
            let namespace = ctx
                .flag("namespace")
                .map(String::as_str)
                .unwrap_or("default");

            // In real kubectl, this would query the K8s API
            let pods = get_pods_in_namespace(namespace);
            Ok(CompletionResult::new()
                .extend(pods.into_iter().filter(|pod| pod.starts_with(prefix))))
        })
        .run(|ctx| {
            let namespace = ctx
                .flag("namespace")
                .map(String::as_str)
                .unwrap_or("default");

            println!("Listing pods in namespace: {namespace}");

            if let Some(pod_name) = ctx.args().first() {
                println!("Getting specific pod: {pod_name}");
            } else {
                for pod in get_pods_in_namespace(namespace) {
                    println!("pod/{}", pod);
                }
            }

            Ok(())
        })
        .build()
}

fn build_get_services() -> Command {
    CommandBuilder::new("services")
        .aliases(vec!["svc", "service"])
        .short("List services")
        .arg_completion(|ctx, prefix| {
            let namespace = ctx
                .flag("namespace")
                .map(String::as_str)
                .unwrap_or("default");

            let services = get_services_in_namespace(namespace);
            Ok(CompletionResult::new()
                .extend(services.into_iter().filter(|svc| svc.starts_with(prefix))))
        })
        .run(|ctx| {
            let namespace = ctx
                .flag("namespace")
                .map(String::as_str)
                .unwrap_or("default");

            println!("Listing services in namespace: {namespace}");
            Ok(())
        })
        .build()
}

fn build_get_deployments() -> Command {
    CommandBuilder::new("deployments")
        .aliases(vec!["deploy", "deployment"])
        .short("List deployments")
        .run(|ctx| {
            let namespace = ctx
                .flag("namespace")
                .map(String::as_str)
                .unwrap_or("default");

            println!("Listing deployments in namespace: {namespace}");
            Ok(())
        })
        .build()
}

fn build_describe_command() -> Command {
    CommandBuilder::new("describe")
        .short("Show details of a specific resource")
        .run(|_ctx| {
            println!("Describe command - add resource type subcommands");
            Ok(())
        })
        .build()
}

fn build_delete_command() -> Command {
    CommandBuilder::new("delete")
        .short("Delete resources")
        .run(|_ctx| {
            println!("Delete command - add resource type subcommands");
            Ok(())
        })
        .build()
}

// Mock functions that would normally query the Kubernetes API
fn get_namespaces() -> Vec<String> {
    vec![
        "default".to_string(),
        "kube-system".to_string(),
        "kube-public".to_string(),
        "development".to_string(),
        "staging".to_string(),
        "production".to_string(),
    ]
}

fn get_pods_in_namespace(namespace: &str) -> Vec<String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate random suffix based on current time
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simple pseudo-random generation
    let rand1 = ((timestamp * 1_103_515_245 + 12345) / 65536) % 100_000;
    let rand2 = ((rand1 * 1_103_515_245 + 12345) / 65536) % 100_000;
    let rand3 = ((rand2 * 1_103_515_245 + 12345) / 65536) % 100_000;

    match namespace {
        "default" => vec![
            format!("nginx-deployment-7fb96c846b-{:05x}", rand1),
            format!("nginx-deployment-7fb96c846b-{:05x}", rand2),
            format!("redis-master-0"),
            format!("redis-slave-0"),
            format!("redis-slave-1"),
        ],
        "kube-system" => vec![
            format!("coredns-5d78c9869d-{:05x}", rand1),
            format!("coredns-5d78c9869d-{:05x}", rand2),
            format!("etcd-minikube"),
            format!("kube-apiserver-minikube"),
            format!("kube-controller-manager-minikube"),
            format!("kube-proxy-{:05x}", rand3),
            format!("kube-scheduler-minikube"),
        ],
        _ => vec![],
    }
}

fn get_services_in_namespace(namespace: &str) -> Vec<String> {
    match namespace {
        "default" => vec![
            "kubernetes".to_string(),
            "nginx-service".to_string(),
            "redis-master".to_string(),
            "redis-slave".to_string(),
        ],
        "kube-system" => vec!["kube-dns".to_string()],
        _ => vec![],
    }
}

fn build_completion_command() -> Command {
    CommandBuilder::new("completion")
        .short("Generate shell completion scripts")
        .long("Generate shell completion scripts for kubectl")
        .run(|ctx| {
            let shell_name = ctx.args().first().ok_or_else(|| {
                flag_rs::Error::ArgumentParsing(
                    "shell name required (bash, zsh, or fish)".to_string(),
                )
            })?;

            let shell = match shell_name.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                _ => {
                    return Err(flag_rs::Error::ArgumentParsing(format!(
                        "unsupported shell: {}",
                        shell_name
                    )));
                }
            };

            // In a real app, you'd get the root command from a shared reference
            // For this example, we'll recreate it
            let root = build_kubectl();
            println!("{}", root.generate_completion(shell));

            Ok(())
        })
        .build()
}
