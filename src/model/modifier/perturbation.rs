use crate::model::modifier::CLIModifier;
use crate::model::LQModel;
use crate::model::actions::ArgumentDescr;

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
    Box::new(CLIPerturbation{})
}

impl CLIModifier for CLIPerturbation {
    fn argument(&self) -> &'static ArgumentDescr {
        &ARGUMENT
    }

    fn modify(&self, mut model: LQModel, parameters: &[&str]) -> LQModel {
        for arg in parameters {
            model = model.perturbation(arg);
        }
        model
    }
}
