use std::collections::HashMap;
use crate::model;
use crate::services::*;
use clap::App;
use crate::model::LQModel;


pub mod show;
pub mod export;
pub mod primes;

lazy_static! {
    pub static ref ACTIONS: ActionManager = ActionManager::new();
}

pub struct ActionManager {
    pub services: HashMap<String, Box<dyn CLIAction>>,
}

impl ActionManager {
    /// Init function to load all available actions
    fn new() -> ActionManager {
        ActionManager {
            services: HashMap::new()
        }
            .register(show::cli_action())
            .register(export::cli_action())
            .register(primes::cli_action())
    }

    fn register(mut self, action: Box<dyn CLIAction>) -> Self {
        self.services.insert(String::from(action.name()), action);
        self
    }
}


pub trait ActionBuilder {
    fn set_flag(&self, flag: &str) {}
    fn set_value(&self, key: &str, value: &str) {}

    fn call(&self);
}

pub trait CLIAction: Sync {
    fn name(&self) -> &'static str;
    fn register_command(&self, mut app: App<'static,'static>) -> App<'static,'static>;
    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder>;
}


pub fn register_commands(mut app: App<'static,'static>) -> App<'static,'static> {
    for cli in ACTIONS.services.values() {
        app = cli.register_command(app)
    }
    app
}


pub fn run_command(cmd: &str, model: LQModel) {

    if let Some(cli) = ACTIONS.services.get(cmd) {
        cli.builder(model).call();
    } else {

    }

}
