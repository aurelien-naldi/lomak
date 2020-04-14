use crate::command::CommandManager;

pub mod buffer;
pub mod perturbation;
pub mod rename;

lazy_static! {
    pub static ref MODIFIERS: CommandManager = CommandManager::new()
        .register(buffer::cli_modifier())
        .register(rename::cli_modifier())
        .register(perturbation::cli_modifier());
}
