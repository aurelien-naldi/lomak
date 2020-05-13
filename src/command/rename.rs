use crate::command::{CLICommand, CommandContext};
use crate::model::LQModelRef;

use std::ffi::OsString;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "rename";
static ABOUT: &str = "Rename one or several components";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// The original name
    source: String,

    /// The target name
    target: String,
}

pub struct CLI;

impl CLICommand for CLI {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        // Start by parsing arguments to handle help without any context
        let config: Config = Config::from_iter(args);

        let mut model = context.as_model();

        // TODO: multiple rename actions ?
        model.rename(&config.source, config.target);

        CommandContext::Model(model)
    }
}
