use crate::model::actions::ActionBuilder;

use crate::model::actions::CLIAction;
use crate::model::LQModel;

struct CLIShow;

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIShow {})
}

impl CLIAction for CLIShow {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show the current model"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["display", "print"]
    }

    fn builder(&self, model: LQModel) -> Box<ActionBuilder> {
        Box::new(ShowBuilder::new(model))
    }
}

pub struct ShowBuilder {
    model: LQModel,
}

impl ShowBuilder {
    pub fn new(model: LQModel) -> ShowBuilder {
        ShowBuilder { model: model }
    }
}

impl ActionBuilder for ShowBuilder {
    fn call(&self) {
        println!("{}", self.model);
    }
}
