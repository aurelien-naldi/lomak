use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::model::actions::stable::FixedBuilder;

static NAME: &str = "fixpoints";
static ABOUT: &str = "Compute the fixed points of the model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Select output components
    displayed: Vec<String>,
}

pub struct CLI;
impl CLICommand for CLI {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn aliases(&self) -> &[&'static str] {
        &["fixed", "stable", "fp"]
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let config: Config = Config::from_iter(args);

        let model = context.as_model();
        let mut builder = FixedBuilder::new(model.as_ref());

        builder.set_displayed_names(config.displayed.iter().map(|s| s.as_str()).collect());
        builder.call();

        CommandContext::Model(model)
    }
}
