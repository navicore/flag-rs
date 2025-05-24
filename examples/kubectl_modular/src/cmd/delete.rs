use flag::{Command, CommandBuilder, Flag, FlagType, FlagValue};

pub fn register(parent: &mut Command) {
    let cmd = CommandBuilder::new("delete")
        .short("Delete resources by filenames, stdin, resources and names, or by resources and label selector")
        .long("Delete resources by filenames, stdin, resources and names, or by resources and label selector. JSON and YAML formats are accepted.")
        .flag(
            Flag::new("force")
                .short('f')
                .usage("Delete immediately, bypassing graceful deletion")
                .value_type(FlagType::Bool)
                .default(FlagValue::Bool(false))
        )
        .flag(
            Flag::new("grace-period")
                .usage("Period of time in seconds given to the resource to terminate gracefully")
                .value_type(FlagType::Int)
                .default(FlagValue::Int(30))
        )
        .flag(
            Flag::new("now")
                .usage("If true, resources are signaled for immediate shutdown")
                .value_type(FlagType::Bool)
                .default(FlagValue::Bool(false))
        )
        .run(|ctx| {
            if ctx.args().is_empty() {
                eprintln!("Error: You must specify the type of resource to delete");
                return Err(flag::Error::ArgumentParsing("resource type required".to_string()));
            }
            
            let resource_type = &ctx.args()[0];
            let resource_name = ctx.args().get(1);
            
            let force = ctx.flag("force")
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false);
            
            let grace_period = ctx.flag("grace-period")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(30);
            
            match resource_type.as_str() {
                "pod" | "pods" | "po" => delete_pod(resource_name, force, grace_period),
                "service" | "services" | "svc" => delete_service(resource_name),
                "deployment" | "deployments" | "deploy" => delete_deployment(resource_name, force),
                _ => {
                    eprintln!("Error: Unknown resource type: {}", resource_type);
                    Err(flag::Error::ArgumentParsing(format!("unknown resource type: {}", resource_type)))
                }
            }
        })
        .build();

    parent.add_command(cmd);
}

fn delete_pod(name: Option<&String>, force: bool, grace_period: i64) -> flag::Result<()> {
    match name {
        Some(pod_name) => {
            if force {
                println!("pod \"{}\" force deleted", pod_name);
            } else {
                println!(
                    "pod \"{}\" deleted (grace period: {}s)",
                    pod_name, grace_period
                );
            }
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

fn delete_service(name: Option<&String>) -> flag::Result<()> {
    match name {
        Some(svc_name) => {
            println!("service \"{}\" deleted", svc_name);
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

fn delete_deployment(name: Option<&String>, force: bool) -> flag::Result<()> {
    match name {
        Some(deploy_name) => {
            if force {
                println!("deployment.apps \"{}\" force deleted", deploy_name);
            } else {
                println!("deployment.apps \"{}\" deleted", deploy_name);
            }
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
