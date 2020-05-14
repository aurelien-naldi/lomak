use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::model::actions::fixpoints::FixedBuilder;

static NAME: &str = "fixpoints";
static ABOUT: &str = "Compute the fixed points of the model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Maximal number of results
    #[structopt(short, long)]
    max: Option<usize>,

    /// Select output components
    #[structopt(short, long)]
    displayed: Option<Vec<String>>,
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
        let builder = FixedBuilder::new(model.as_ref());
        let mut result = builder.solve(config.max);

        if let Some(display) = config.displayed {
            result.set_displayed_names(Some(display));
        }
        println!("{}", result);

        CommandContext::Model(model)
    }
}
