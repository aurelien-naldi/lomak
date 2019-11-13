use regex::Regex;

use crate::model::actions::ArgumentDescr;
use crate::model::modifier::CLIModifier;
use crate::model::{LQModelRef, QModel};
use std::borrow::BorrowMut;

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
    pub static ref ARGUMENT: ArgumentDescr = ArgumentDescr::new("prt")
        .long("perturbation")
        .short("p")
        .has_value(true)
        .multiple(true)
        .help("Apply a perturbation to one or several components");
}

pub struct CLIPerturbation;

pub fn cli_modifier() -> Box<dyn CLIModifier> {
    Box::new(CLIPerturbation {})
}

impl CLIModifier for CLIPerturbation {
    fn argument(&self) -> &'static ArgumentDescr {
        &ARGUMENT
    }

    fn modify(&self, mut rmodel: LQModelRef, parameters: &[&str]) -> LQModelRef {
        let model: &mut dyn QModel = rmodel.borrow_mut();
        for arg in parameters {
            match RE_PRT.captures(arg) {
                None => println!("Invalid perturbation parameter: {}", arg),
                Some(cap) => {
                    if let Some(uid) = model.component_by_name(&cap[1]) {
                        match &cap[2] {
                            "0" => model.lock(uid, false),
                            "1" => model.lock(uid, true),
                            _ => println!("Invalid target value: {}", &cap[2]),
                        }
                    }
                }
            }
        }
        rmodel
    }
}
