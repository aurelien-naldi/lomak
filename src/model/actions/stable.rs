use crate::model;
use crate::model::io;
use clap::App;
use clap::SubCommand;
use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;


pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed{})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str { "fixpoints" }

    fn register_command(&self, mut app: App<'static, 'static>) -> App<'static, 'static> {
        app.subcommand(SubCommand::with_name(self.name())
            .about("Compute the fixed points of the model")
            .aliases(&["fixed", "stable"])
        )
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(FixedBuilder::new(model))
    }
}


pub struct FixedBuilder{
    model: LQModel,
}


impl FixedBuilder {
    pub fn new(model: LQModel) -> FixedBuilder {
        FixedBuilder{model: model}
    }
}

impl ActionBuilder for FixedBuilder {

    fn call(&self) {
        self.model.stable_full(true);
    }
}
