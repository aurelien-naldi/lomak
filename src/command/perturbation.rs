use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::helper::error::EmptyLomakResult;
use crate::model::modifier::perturbation::Perturbator;
use std::ops::DerefMut;

static NAME: &str = "perturbation";
static ABOUT: &str = "Apply a perturbation to one or several components or interactions (source@target)";

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

        // Prepare the perturbator
        let smodel = context.get_model()?;
        let mut model = smodel.borrow_mut();
        let mut perturbator = Perturbator::new(model.deref_mut());

        // Prepare the list of all perturbations
        for s in &config.ko {
            perturbator.guess_lock(s, false)?;
        }
        for s in &config.ki {
            perturbator.guess_lock(s, true)?;
        }

        // Apply all perturbations
        perturbator.apply();
        Ok(())
    }
}
