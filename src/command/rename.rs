use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::error::EmptyLomakResult;
use crate::variables::GroupedVariables;
use std::ops::DerefMut;

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

    fn run(&self, context: &mut CommandContext, args: &[OsString]) -> EmptyLomakResult {
        // Start by parsing arguments to handle help without any context
        let config: Config = Config::from_iter(args);

        let smodel = context.get_model()?;

        // TODO: multiple rename actions ?
        let mut model = smodel.borrow_mut();
        model.deref_mut().rename(&config.source, &config.target);

        Ok(())
    }
}
