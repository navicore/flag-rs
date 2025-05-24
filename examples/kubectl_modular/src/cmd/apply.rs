use flag_rs::{Command, CommandBuilder, Flag, FlagType, FlagValue};

pub fn register(parent: &mut Command) {
    let cmd = CommandBuilder::new("apply")
        .short("Apply a configuration to a resource by filename or stdin")
        .long("Apply a configuration to a resource by filename or stdin. The resource name must be specified. This resource will be created if it doesn't exist yet.")
        .flag(
            Flag::new("filename")
                .short('f')
                .usage("Filename, directory, or URL to files to use to apply the configuration")
                .value_type(FlagType::String)
                .required()
        )
        .flag(
            Flag::new("dry-run")
                .usage("If true, only print the object that would be sent, without sending it")
                .value_type(FlagType::Bool)
                .default(FlagValue::Bool(false))
        )
        .flag(
            Flag::new("recursive")
                .short('R')
                .usage("Process the directory used in -f, --filename recursively")
                .value_type(FlagType::Bool)
                .default(FlagValue::Bool(false))
        )
        .run(|ctx| {
            let filename = match ctx.flag("filename") {
                Some(f) => f,
                None => {
                    eprintln!("Error: Must specify --filename/-f");
                    return Err(flag_rs::Error::ArgumentParsing("filename required".to_string()));
                }
            };

            let dry_run = ctx.flag("dry-run")
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false);

            let recursive = ctx.flag("recursive")
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false);

            if dry_run {
                println!("(dry run) Applying configuration from: {}", filename);
            } else {
                println!("Applying configuration from: {}", filename);
            }

            if recursive {
                println!("Processing directory recursively...");
            }

            // Simulate applying different resource types
            if filename.ends_with("deployment.yaml") {
                println!("deployment.apps/nginx-deployment configured");
            } else if filename.ends_with("service.yaml") {
                println!("service/nginx-service configured");
            } else if filename.ends_with("configmap.yaml") {
                println!("configmap/app-config configured");
            } else {
                println!("resource configured");
            }

            Ok(())
        })
        .build();

    parent.add_command(cmd);
}
