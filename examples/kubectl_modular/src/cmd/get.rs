use flag::{Command, CommandBuilder};

pub fn register(parent: &mut Command) {
    let mut cmd = CommandBuilder::new("get")
        .short("Display one or many resources")
        .long("Display one or many resources. Prints a table of the most important information about the specified resources.")
        .build();

    // Register subcommands
    pods::register(&mut cmd);
    services::register(&mut cmd);
    deployments::register(&mut cmd);

    parent.add_command(cmd);
}

// Create a submodule for pods
mod pods {
    use flag::{Command, CommandBuilder, CompletionResult};

    pub fn register(parent: &mut Command) {
        let cmd = CommandBuilder::new("pods")
            .aliases(vec!["po", "pod"])
            .short("List pods")
            .arg_completion(|ctx, prefix| {
                let namespace = ctx
                    .flag("namespace")
                    .map(|s| s.as_str())
                    .unwrap_or("default");

                let pods = get_pods_in_namespace(namespace);
                Ok(CompletionResult::new().extend(
                    pods.into_iter()
                        .filter(|pod| pod.starts_with(prefix))
                        .collect::<Vec<_>>(),
                ))
            })
            .run(|ctx| {
                let namespace = ctx
                    .flag("namespace")
                    .map(|s| s.as_str())
                    .unwrap_or("default");

                println!(
                    "NAME                                    READY   STATUS    RESTARTS   AGE"
                );

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
        let rand1 = ((timestamp * 1103515245 + 12345) / 65536) % 100000;
        let rand2 = ((rand1 * 1103515245 + 12345) / 65536) % 100000;
        let rand3 = ((rand2 * 1103515245 + 12345) / 65536) % 100000;

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
}

// Create a submodule for services
mod services {
    use flag::{Command, CommandBuilder, CompletionResult};

    pub fn register(parent: &mut Command) {
        let cmd = CommandBuilder::new("services")
            .aliases(vec!["svc", "service"])
            .short("List services")
            .arg_completion(|ctx, prefix| {
                let namespace = ctx
                    .flag("namespace")
                    .map(|s| s.as_str())
                    .unwrap_or("default");

                let services = get_services_in_namespace(namespace);
                Ok(CompletionResult::new().extend(
                    services
                        .into_iter()
                        .filter(|svc| svc.starts_with(prefix))
                        .collect::<Vec<_>>(),
                ))
            })
            .run(|ctx| {
                let namespace = ctx
                    .flag("namespace")
                    .map(|s| s.as_str())
                    .unwrap_or("default");

                println!(
                    "NAME              TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)    AGE"
                );

                for service in get_services_in_namespace(namespace) {
                    println!(
                        "{:<17} ClusterIP   10.96.0.1       <none>        443/TCP    30d",
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
            "kube-system" => vec!["kube-dns".to_string()],
            _ => vec![],
        }
    }
}

// Create a submodule for deployments
mod deployments {
    use flag::{Command, CommandBuilder};

    pub fn register(parent: &mut Command) {
        let cmd = CommandBuilder::new("deployments")
            .aliases(vec!["deploy", "deployment"])
            .short("List deployments")
            .run(|ctx| {
                let namespace = ctx
                    .flag("namespace")
                    .map(|s| s.as_str())
                    .unwrap_or("default");

                println!("NAME               READY   UP-TO-DATE   AVAILABLE   AGE");
                if namespace == "default" {
                    println!("nginx-deployment   2/2     2            2           3d");
                    println!("redis-master       1/1     1            1           5d");
                    println!("redis-slave        2/2     2            2           5d");
                }

                Ok(())
            })
            .build();

        parent.add_command(cmd);
    }
}
