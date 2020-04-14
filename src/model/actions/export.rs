use crate::command::{CLICommand, CommandContext};
use crate::model::io;
use crate::model::QModel;

use std::borrow::Borrow;
use std::ffi::OsString;
use std::sync::Arc;
use structopt::StructOpt;

static NAME: &str = "export";
static ABOUT: &str = "Save the current model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct ExportConfig {
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

    fn help(&self) {
        ExportConfig::clap().print_help();
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["save", "convert"]
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let model = match &context {
            CommandContext::Model(m) => m,
            _ => panic!("invalid context"),
        };

        let config: ExportConfig = ExportConfig::from_iter(args);

        io::save_model(model.borrow(), &config.output, config.format.as_deref());

        // TODO: should it return the model or an empty context?
        context
    }
}
