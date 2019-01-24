use crate::services::*;

use crate::model;

pub trait Action: Service {
    fn run(&self, model: &model::LQModel);
}

pub struct ActionService {
    _info: ServiceInfo,
    _callback: Box<Fn(&model::LQModel)>,
}

impl Service for ActionService {
    fn info(&self) -> &ServiceInfo {
        &self._info
    }
}
impl Action for ActionService {
    fn run(&self, model: &model::LQModel) {
        self._callback.as_ref()(model)
    }
}

/// Init function to load all available actions
pub fn load_actions() -> ServiceManager<ActionService> {
    ServiceManager::new()
        .register(show_action())
        .register(primes_action())
        .register(export_action())
        .register(fixpoints_action())
}

/// Private function to create the "show" action
fn show_action() -> ActionService {
    ActionService {
        _info: ServiceInfo::new("show").descr("Show the current model"),
        _callback: Box::from(run_show_action),
    }
}

fn run_show_action(model: &model::LQModel) {
    print!("{}", model);
}

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
