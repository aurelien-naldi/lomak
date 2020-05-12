use crate::command::{CLICommand, CommandContext};
use crate::model::LQModelRef;

use std::ffi::OsString;
use std::sync::Arc;
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

pub struct CLIRename;

pub fn cli_modifier() -> Arc<dyn CLICommand> {
    Arc::new(CLIRename {})
}

impl CLICommand for CLIRename {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn clap(&self) -> App {
        Config::clap()
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = context.as_model();
        let config: Config = Config::from_iter(args);

        // TODO: multiple rename actions ?
        model.rename(&config.source, config.target);

        CommandContext::Model(model)
    }
}
