use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::helper::error::EmptyLomakResult;
use itertools::Itertools;

static NAME: &str = "perturbation";
static ABOUT: &str = "Apply a perturbation to one or several components";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Components to knock-out (fix level to 0)
    #[structopt(long)]
    ko: Vec<String>,

    /// Components to knock-in (fix level to 1)
    #[structopt(long)]
    ki: Vec<String>,
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

        // assemble all parameters into a single pairing iterator
        let kos = config.ko.iter().map(|n| (&**n, false));
        let kis = config.ki.iter().map(|n| (&**n, true));
        // apply all perturbations
        smodel.lock(kos.merge(kis))?;
        Ok(())
    }
}
