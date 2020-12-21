use std::ffi::OsString;
use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::helper::error::EmptyLomakResult;

static NAME: &str = "save";
static ABOUT: &str = "Save the current model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Set the output file
    output: String,

    /// Enforce the output format
    #[structopt(short = "F", long)]
    format: Option<String>,
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
        &["export", "convert"]
    }

    fn run(&self, context: &mut CommandContext, args: &[OsString]) -> EmptyLomakResult {
        let config: Config = Config::from_iter(args);
        context
            .get_model()?
            .save(&config.output, config.format.as_deref())?;

        Ok(())
    }
}
