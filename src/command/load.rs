use crate::command::{CLICommand, CommandContext};
use crate::model::LQModelRef;
use crate::model::io;

use std::ffi::OsString;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "load";
static ABOUT: &str = "Load a model from a file";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    #[structopt(short="F", long)]
    format: Option<String>,
    filename: String
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

        let model = match io::load_model(&config.filename, config.format.as_deref()) {
            Err(e) => {
                println!("ERROR loading \"{}\": {}", &config.filename, e);
                return CommandContext::Empty;
            }
            Ok(m) => m,
        };

        CommandContext::Model(model)
    }
}
