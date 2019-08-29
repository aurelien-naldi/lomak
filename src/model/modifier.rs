use crate::model::actions::{arg_from_descr, ArgumentDescr};
use crate::model::LQModelRef;
use clap::{App, ArgMatches};
use std::collections::HashMap;

pub mod perturbation;
pub mod rename;

lazy_static! {
    pub static ref MODIFIERS: ModifierManager = ModifierManager::new();
}

pub struct ModifierManager {
    modifiers: HashMap<String, Box<dyn CLIModifier>>,
}

pub trait CLIModifier: Sync {
    fn argument(&self) -> &'static ArgumentDescr;

    fn modify(&self, model: LQModelRef, parameters: &[&str]) -> LQModelRef;
}

impl ModifierManager {
    /// Init function to load all available actions
    fn new() -> ModifierManager {
        ModifierManager {
            modifiers: HashMap::new(),
        }
        .modifier(perturbation::cli_modifier())
        .modifier(rename::cli_modifier())
    }

    fn modifier(mut self, modifier: Box<dyn CLIModifier>) -> Self {
        self.modifiers
            .insert(modifier.argument().name.clone(), modifier);
        self
    }

    fn modify(&self, mut model: LQModelRef, matches: &ArgMatches) -> LQModelRef {
        // TODO: apply selected modifiers in the right order
        for (name, cli) in &self.modifiers {
            if matches.is_present(name) {
                if let Some(params) = matches.values_of(name) {
                    let p: Vec<&str> = params.map(|v| v.trim()).collect();
                    model = cli.modify(model, p.as_slice());
                } else {
                    model = cli.modify(model, &[]);
                }
            }
        }
        model
    }
}

pub fn register_modifiers(mut app: App<'static, 'static>) -> App<'static, 'static> {
    for cli in MODIFIERS.modifiers.values() {
        app = app.arg(arg_from_descr(cli.argument()))
    }
    app
}

pub fn modify(model: LQModelRef, matches: &ArgMatches) -> LQModelRef {
    MODIFIERS.modify(model, matches)
}

pub trait ModelModifier {
    fn get_model(self) -> LQModelRef;
}
