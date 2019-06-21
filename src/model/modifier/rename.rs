use crate::func::variables::VariableNamer;
use crate::model::actions::ArgumentDescr;
use crate::model::modifier::CLIModifier;
use crate::model::LQModel;

lazy_static! {
    pub static ref ARGUMENT: ArgumentDescr = ArgumentDescr::new("mv")
        .long("mv")
        .short("m")
        .has_value(true)
        .multiple(true)
        .help("Rename one or several components");
}

pub struct CLIRename;

pub fn cli_modifier() -> Box<dyn CLIModifier> {
    Box::new(CLIRename {})
}

impl CLIModifier for CLIRename {
    fn argument(&self) -> &'static ArgumentDescr {
        &ARGUMENT
    }

    fn modify(&self, mut model: LQModel, parameters: &[&str]) -> LQModel {
        for arg in parameters {
            let split: Vec<&str> = arg.split(":").collect();
            if split.len() != 2 {
                println!("invalid rename pattern");
                continue;
            }
            model.rename(split[0], format!("{}", split[1]));
        }
        model
    }
}
