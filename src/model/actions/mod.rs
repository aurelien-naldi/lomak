use std::collections::HashMap;
use crate::model;
use crate::services::*;


pub mod show;
pub mod export;

pub trait Action: Service {
    fn run(&self, model: &model::LQModel);
}

lazy_static! {
    pub static ref ACTIONS: ActionManager = ActionManager::new();
}

pub struct ActionManager {
    pub services: HashMap<String, Box<dyn Action>>,
}

impl ActionManager {

    /// Init function to load all available actions
    fn new() -> ActionManager {

        ActionManager {
            services: HashMap::new()
        }
            .register(Box::new(show::ShowService{} ))
            .register(Box::new(export::ExportService{} ))
//        .register(primes_action())
//        .register(fixpoints_action())
    }

    fn register(mut self, srv: Box<dyn Action>) -> Self {
        self.services.insert(String::from(srv.name()), srv);
        self
    }

    pub fn actions(&self) { // -> std::collections::hash_map::Values {
        let v = self.services.values();
    }

}

/*
fn run_primes_action(model: &model::LQModel) {
    model.primes();
}
fn run_stable_action(model: &model::LQModel) {
    model.stable_full(true);
}
fn run_export_action(model: &model::LQModel) {
    println!("TODO: export the model");
}

/// Private function to create the "primes" action
fn primes_action() -> ActionService {
    ActionService {
        _info: ServiceInfo::new("primes")
            .alias("pi")
            .descr("Show the Prime Implicants"),
        _callback: Box::from(run_primes_action),
    }
}

/// Private function to create the "show" action
fn fixpoints_action() -> ActionService {
    ActionService {
        _info: ServiceInfo::new("fixpoints")
            .alias("fp")
            .alias("stables")
            .descr("[TODO] Find fixed points"),
        _callback: Box::from(run_stable_action),
    }
}

/// Private function to create the "export" action
fn export_action() -> ActionService {
    ActionService {
        _info: ServiceInfo::new("export")
            .alias("save")
            .alias("convert")
            .descr("[TODO] Save the (modified) model")
            .argument(
                Argument::new("format")
                    .descr("Set the export format")
                    .short("F")
                    .long("format")
                    .value(true),
            ),
        _callback: Box::from(run_export_action),
    }
}

*/

