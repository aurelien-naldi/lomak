use crate::command::{CLICommand, CommandContext};
use crate::model::{io, LQModelRef};
use crate::model::QModel;

use std::borrow::Borrow;
use std::ffi::OsString;
use std::sync::Arc;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "export";
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

pub fn cli_action() -> Arc<dyn CLICommand> {
    Arc::new(CLIExport {})
}

struct CLIExport;

impl CLICommand for CLIExport {

    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn clap(&self) -> App {
        Config::clap()
    }

    fn aliases(&self) -> &[&'static str] {
        &["save", "convert"]
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = context.as_model();
        let config: Config = Config::from_iter(args);

        // Save the model
        io::save_model(model.borrow(), &config.output, config.format.as_deref());

        CommandContext::Model(model)
    }
}
