use flag::{Command, CommandBuilder};

pub fn register(parent: &mut Command) {
    let cmd = CommandBuilder::new("describe")
        .short("Show details of a specific resource or group of resources")
        .long("Show details of a specific resource or group of resources. Print a detailed description of the selected resources, including related resources such as events or controllers.")
        .run(|ctx| {
            if ctx.args().is_empty() {
                eprintln!("Error: You must specify the type of resource to describe. Use \"kubectl api-resources\" for a complete list of supported resources.");
                return Err(flag::Error::ArgumentParsing("resource type required".to_string()));
            }

            let resource_type = &ctx.args()[0];
            let resource_name = ctx.args().get(1);

            match resource_type.as_str() {
                "pod" | "pods" | "po" => describe_pod(resource_name),
                "service" | "services" | "svc" => describe_service(resource_name),
                "deployment" | "deployments" | "deploy" => describe_deployment(resource_name),
                _ => {
                    eprintln!("Error: Unknown resource type: {}", resource_type);
                    Err(flag::Error::ArgumentParsing(format!("unknown resource type: {}", resource_type)))
                }
            }
        })
        .build();

    parent.add_command(cmd);
}

fn describe_pod(name: Option<&String>) -> flag::Result<()> {
    match name {
        Some(pod_name) => {
            println!("Name:         {}", pod_name);
            println!("Namespace:    default");
            println!("Priority:     0");
            println!("Node:         minikube/192.168.99.100");
            println!("Start Time:   Mon, 15 Jan 2024 10:30:45 +0000");
            println!("Labels:       app=nginx");
            println!("              pod-template-hash=7fb96c846b");
            println!("Status:       Running");
            println!("IP:           172.17.0.4");
            println!();
            println!("Containers:");
            println!("  nginx:");
            println!("    Container ID:   docker://abc123...");
            println!("    Image:          nginx:1.14.2");
            println!("    Port:           80/TCP");
            println!("    State:          Running");
            println!("      Started:      Mon, 15 Jan 2024 10:31:00 +0000");
            println!("    Ready:          True");
            println!();
            println!("Events:");
            println!("  Type    Reason     Age   From               Message");
            println!("  ----    ------     ----  ----               -------");
            println!(
                "  Normal  Scheduled  2d    default-scheduler  Successfully assigned default/{} to minikube",
                pod_name
            );
            println!(
                "  Normal  Pulled     2d    kubelet            Container image \"nginx:1.14.2\" already present on machine"
            );
            println!("  Normal  Created    2d    kubelet            Created container nginx");
            println!("  Normal  Started    2d    kubelet            Started container nginx");
            Ok(())
        }
        None => {
            eprintln!("Error: You must specify a pod name");
            Err(flag::Error::ArgumentParsing(
                "pod name required".to_string(),
            ))
        }
    }
}

fn describe_service(name: Option<&String>) -> flag::Result<()> {
    match name {
        Some(svc_name) => {
            println!("Name:              {}", svc_name);
            println!("Namespace:         default");
            println!("Labels:            app=nginx");
            println!("Annotations:       <none>");
            println!("Selector:          app=nginx");
            println!("Type:              ClusterIP");
            println!("IP Family Policy:  SingleStack");
            println!("IP Families:       IPv4");
            println!("IP:                10.96.0.1");
            println!("IPs:               10.96.0.1");
            println!("Port:              <unset>  80/TCP");
            println!("TargetPort:        80/TCP");
            println!("Endpoints:         172.17.0.4:80,172.17.0.5:80");
            println!("Session Affinity:  None");
            println!("Events:            <none>");
            Ok(())
        }
        None => {
            eprintln!("Error: You must specify a service name");
            Err(flag::Error::ArgumentParsing(
                "service name required".to_string(),
            ))
        }
    }
}

fn describe_deployment(name: Option<&String>) -> flag::Result<()> {
    match name {
        Some(deploy_name) => {
            println!("Name:                   {}", deploy_name);
            println!("Namespace:              default");
            println!("CreationTimestamp:      Mon, 15 Jan 2024 10:30:00 +0000");
            println!("Labels:                 app=nginx");
            println!("Annotations:            deployment.kubernetes.io/revision: 1");
            println!("Selector:               app=nginx");
            println!(
                "Replicas:               2 desired | 2 updated | 2 total | 2 available | 0 unavailable"
            );
            println!("StrategyType:           RollingUpdate");
            println!("MinReadySeconds:        0");
            println!("RollingUpdateStrategy:  25% max unavailable, 25% max surge");
            println!("Pod Template:");
            println!("  Labels:  app=nginx");
            println!("  Containers:");
            println!("   nginx:");
            println!("    Image:        nginx:1.14.2");
            println!("    Port:         80/TCP");
            println!("    Environment:  <none>");
            println!("    Mounts:       <none>");
            println!("  Volumes:        <none>");
            println!("Conditions:");
            println!("  Type           Status  Reason");
            println!("  ----           ------  ------");
            println!("  Available      True    MinimumReplicasAvailable");
            println!("  Progressing    True    NewReplicaSetAvailable");
            Ok(())
        }
        None => {
            eprintln!("Error: You must specify a deployment name");
            Err(flag::Error::ArgumentParsing(
                "deployment name required".to_string(),
            ))
        }
    }
}
