use crate::model;
use crate::services::*;
use crate::model::actions::ActionBuilder;

use clap::SubCommand;
use clap::App;
use crate::model::LQModel;
use crate::model::actions::CLIAction;


struct CLIShow;

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIShow{})
}

impl CLIAction for CLIShow {
    fn name(&self) -> &'static str { "show" }

    fn register_command<'a,'b>(&self, mut app: App<'a,'b>) -> App<'a,'b> {
        app.subcommand(SubCommand::with_name(self.name())
            .about("Show the current model")
            .aliases(&["display", "print"])
        )
    }

    fn builder(&self, model: LQModel) -> Box<ActionBuilder> {
        Box::new(ShowBuilder::new(model) )
    }
}


pub struct ShowBuilder {
    model: LQModel,
}

impl ShowBuilder {
    pub fn new(model: LQModel) -> ShowBuilder {
        ShowBuilder{model: model}
    }
}

impl ActionBuilder for ShowBuilder {

    fn call(&self) {
        println!("{}", self.model);
    }
}
