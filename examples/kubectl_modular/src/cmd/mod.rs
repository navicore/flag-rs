use flag_rs::Command;

mod apply;
mod completion;
mod delete;
mod describe;
mod get;

pub fn register_commands(root: &mut Command) {
    // Each subcommand module registers itself
    get::register(root);
    describe::register(root);
    delete::register(root);
    apply::register(root);
    completion::register(root);
}
