use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::helper::error::EmptyLomakResult;
use crate::model::io;

static NAME: &str = "load";
static ABOUT: &str = "Load a model from a file";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Enforce format instead of using file extension
    #[structopt(short = "F", long)]
    format: Option<String>,

    /// File containing the model
    filename: String,
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

        let model = io::load_model(&config.filename, config.format.as_deref())?;
        context.set_model(model, None);
        Ok(())
    }
}
