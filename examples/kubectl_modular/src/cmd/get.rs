use flag::{CommandBuilder, CompletionResult};

pub fn register(parent: &mut flag::Command) {
    // Main get command
    let cmd = CommandBuilder::new("get")
        .short("Display one or many resources")
        .long("Display one or many resources. Prints a table of the most important information about the specified resources.")
        .build();

    parent.add_command(cmd);

    // Register subcommands
    register_pods(parent.find_subcommand_mut("get").unwrap());
    register_services(parent.find_subcommand_mut("get").unwrap());
    register_deployments(parent.find_subcommand_mut("get").unwrap());
}

fn register_pods(parent: &mut flag::Command) {
    let cmd = CommandBuilder::new("pods")
        .aliases(vec!["po", "pod"])
        .short("List pods")
        .arg_completion(|ctx, prefix| {
            // This is the key feature - dynamic completion based on runtime state!
            let namespace = ctx.flag("namespace").map_or("default", String::as_str);

            let pods = get_pods_in_namespace(namespace);
            Ok(CompletionResult::new()
                .extend(pods.into_iter().filter(|pod| pod.starts_with(prefix))))
        })
        .run(|ctx| {
            let namespace = ctx.flag("namespace").map_or("default", String::as_str);

            println!("NAME                                    READY   STATUS    RESTARTS   AGE");

            if let Some(pod_name) = ctx.args().first() {
                // Show specific pod
                println!("{:<40} 1/1     Running   0          2d", pod_name);
            } else {
                // List all pods
                for pod in get_pods_in_namespace(namespace) {
                    println!("{:<40} 1/1     Running   0          2d", pod);
                }
            }

            Ok(())
        })
        .build();

    parent.add_command(cmd);
}

fn get_pods_in_namespace(namespace: &str) -> Vec<String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate random suffix based on current time - proves this is runtime!
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simple pseudo-random generation
    let rand1 = ((timestamp * 1_103_515_245 + 12345) / 65536) % 100_000;
    let rand2 = ((rand1 * 1_103_515_245 + 12345) / 65536) % 100_000;
    let rand3 = ((rand2 * 1_103_515_245 + 12345) / 65536) % 100_000;

    // In a real implementation, this would query the Kubernetes API
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

fn register_services(parent: &mut flag::Command) {
    let cmd = CommandBuilder::new("services")
        .aliases(vec!["svc", "service"])
        .short("List services")
        .arg_completion(|ctx, prefix| {
            let namespace = ctx.flag("namespace").map_or("default", String::as_str);

            let services = get_services_in_namespace(namespace);
            Ok(CompletionResult::new()
                .extend(services.into_iter().filter(|svc| svc.starts_with(prefix))))
        })
        .run(|ctx| {
            let namespace = ctx.flag("namespace").map_or("default", String::as_str);

            println!("NAME         TYPE        CLUSTER-IP       EXTERNAL-IP   PORT(S)    AGE");

            for service in get_services_in_namespace(namespace) {
                println!(
                    "{:<12} ClusterIP   10.96.0.1        <none>        443/TCP    30d",
                    service
                );
            }

            Ok(())
        })
        .build();

    parent.add_command(cmd);
}

fn get_services_in_namespace(namespace: &str) -> Vec<String> {
    match namespace {
        "default" => vec![
            "kubernetes".to_string(),
            "nginx-service".to_string(),
            "redis-master".to_string(),
            "redis-slave".to_string(),
        ],
        "kube-system" => vec!["kube-dns".to_string(), "metrics-server".to_string()],
        _ => vec![],
    }
}

fn register_deployments(parent: &mut flag::Command) {
    let cmd = CommandBuilder::new("deployments")
        .aliases(vec!["deploy", "deployment"])
        .short("List deployments")
        .arg_completion(|ctx, prefix| {
            let namespace = ctx.flag("namespace").map_or("default", String::as_str);

            let deployments = get_deployments_in_namespace(namespace);
            Ok(CompletionResult::new().extend(
                deployments
                    .into_iter()
                    .filter(|dep| dep.starts_with(prefix)),
            ))
        })
        .run(|ctx| {
            let namespace = ctx.flag("namespace").map_or("default", String::as_str);

            println!("NAME               READY   UP-TO-DATE   AVAILABLE   AGE");

            for deployment in get_deployments_in_namespace(namespace) {
                println!("{:<18} 3/3     3            3           5d", deployment);
            }

            Ok(())
        })
        .build();

    parent.add_command(cmd);
}

fn get_deployments_in_namespace(namespace: &str) -> Vec<String> {
    match namespace {
        "default" => vec![
            "nginx-deployment".to_string(),
            "redis-master".to_string(),
            "redis-slave".to_string(),
        ],
        "kube-system" => vec!["coredns".to_string(), "metrics-server".to_string()],
        _ => vec![],
    }
}
