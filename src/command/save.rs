use std::ffi::OsString;
use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};

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

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let config: Config = Config::from_iter(args);
        match context
            .get_model()
            .save(&config.output, config.format.as_deref())
        {
            Ok(_) => (),
            Err(e) => eprintln!("Error during save: {}", e),
        };

        context
    }
}
