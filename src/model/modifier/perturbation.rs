use crate::model::actions::ArgumentDescr;
use crate::model::modifier::CLIModifier;
use crate::model::LQModelRef;

lazy_static! {
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

    fn modify(&self, mut model: LQModelRef, parameters: &[&str]) -> LQModelRef {
        for arg in parameters {
            model.perturbation(arg);
        }
        model
    }
}
