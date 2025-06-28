//! Demonstrates enhanced help formatting
//!
//! This example shows the improved help output with:
//! - Command aliases display
//! - Usage examples
//! - Better spacing and organization

use flag_rs::{CommandBuilder, Flag, FlagType};

fn main() {
    let app = CommandBuilder::new("kubectl")
        .short("Kubernetes command-line tool")
        .long("kubectl controls the Kubernetes cluster manager.\n\n\
              Find more information at: https://kubernetes.io/docs/reference/kubectl/")
        .example("kubectl get pods")
        .example("kubectl apply -f deployment.yaml")
        .example("kubectl logs -f my-pod")
        .flag(
            Flag::new("namespace")
                .short('n')
                .usage("The namespace scope for this CLI request")
                .value_type(FlagType::String)
                .default(flag_rs::FlagValue::String("default".to_string())),
        )
        .flag(
            Flag::new("kubeconfig")
                .usage("Path to the kubeconfig file to use for CLI requests")
                .value_type(FlagType::String),
        )
        .subcommand(
            CommandBuilder::new("get")
                .short("Display one or many resources")
                .group_id("Basic Commands")
                .long("Prints a table of the most important information about the specified resources.\n\n\
                      You can filter the list using a label selector and the --selector flag. If the\n\
                      desired resource type is namespaced you will only see results in your current\n\
                      namespace unless you pass --all-namespaces.")
                .example("kubectl get pods")
                .example("kubectl get pods -n kube-system")
                .example("kubectl get pods --selector=app=nginx")
                .example("kubectl get pods,services")
                .build(),
        )
        .subcommand(
            CommandBuilder::new("apply")
                .short("Apply a configuration to a resource by file name or stdin")
                .group_id("Basic Commands")
                .long("Apply a configuration to a resource by file name or stdin. The resource name must\n\
                      be specified. This resource will be created if it doesn't exist yet. To use 'apply',\n\
                      always create the resource initially with either 'apply' or 'create --save-config'.")
                .example("kubectl apply -f ./pod.yaml")
                .example("kubectl apply -f https://example.com/manifest.yaml")
                .example("kubectl apply -k ./")
                .flag(
                    Flag::new("filename")
                        .short('f')
                        .usage("Filename, directory, or URL to files to use to create the resource")
                        .value_type(FlagType::String)
                        .required(),
                )
                .flag(
                    Flag::new("recursive")
                        .short('R')
                        .usage("Process the directory used in -f, --filename recursively")
                        .value_type(FlagType::Bool),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("delete")
                .short("Delete resources by file names, stdin, resources and names, or by resources and label selector")
                .aliases(vec!["del", "remove", "rm"])
                .group_id("Basic Commands")
                .example("kubectl delete pod my-pod")
                .example("kubectl delete -f ./pod.yaml")
                .example("kubectl delete pods --all")
                .build(),
        )
        .subcommand(
            CommandBuilder::new("logs")
                .short("Print the logs for a container in a pod")
                .group_id("Troubleshooting and Debugging Commands")
                .long("Print the logs for a container in a pod or specified resource. If the pod has\n\
                      only one container, the container name is optional.")
                .aliases(vec!["log"])
                .example("kubectl logs my-pod")
                .example("kubectl logs my-pod -c my-container")
                .example("kubectl logs -f my-pod")
                .example("kubectl logs --tail=20 my-pod")
                .flag(
                    Flag::new("follow")
                        .short('f')
                        .usage("Specify if the logs should be streamed")
                        .value_type(FlagType::Bool),
                )
                .flag(
                    Flag::new("tail")
                        .usage("Lines of recent log file to display")
                        .value_type(FlagType::Int)
                        .default(flag_rs::FlagValue::Int(-1)),
                )
                .build(),
        )
        .subcommand(
            CommandBuilder::new("describe")
                .short("Show details of a specific resource or group of resources")
                .group_id("Troubleshooting and Debugging Commands")
                .example("kubectl describe pod my-pod")
                .example("kubectl describe nodes")
                .build(),
        )
        .subcommand(
            CommandBuilder::new("exec")
                .short("Execute a command in a container")
                .group_id("Troubleshooting and Debugging Commands")
                .example("kubectl exec -it my-pod -- /bin/bash")
                .example("kubectl exec my-pod -- ls /app")
                .build(),
        )
        .subcommand(
            CommandBuilder::new("config")
                .short("Modify kubeconfig files")
                .group_id("Settings Commands")
                .example("kubectl config view")
                .example("kubectl config use-context my-context")
                .build(),
        )
        .subcommand(
            CommandBuilder::new("version")
                .short("Print the client and server version information")
                .example("kubectl version")
                .example("kubectl version --short")
                .build(),
        )
        .build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = app.execute(args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
