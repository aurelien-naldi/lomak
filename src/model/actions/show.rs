use crate::model::actions::ActionBuilder;

use crate::model::actions::CLIAction;
use crate::model::QModel;

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

    fn builder<'a>(&self, model: &'a dyn QModel) -> Box<dyn ActionBuilder + 'a> {
        Box::new(ShowBuilder::new(model))
    }
}

pub struct ShowBuilder<'a> {
    model: &'a dyn QModel,
}

impl<'a> ShowBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> ShowBuilder<'a> {
        ShowBuilder { model }
    }
}

impl ActionBuilder for ShowBuilder<'_> {
    fn call(&self) {
        println!("{}", self.model.for_display());
        println!();
    }
}
