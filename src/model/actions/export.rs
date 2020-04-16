use crate::command::{CLICommand, CommandContext};
use crate::model::{io, LQModelRef};
use crate::model::QModel;

use std::borrow::Borrow;
use std::ffi::OsString;
use std::sync::Arc;
use structopt::StructOpt;
use crate::model::actions::CLIAction;

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

impl CLIAction for CLIExport {
    type Config = ExportConfig;

    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn aliases(&self) -> &[&'static str] {
        &["save", "convert"]
    }

    fn run_model(&self, model: &LQModelRef, config: ExportConfig) {
        io::save_model(model.borrow(), &config.output, config.format.as_deref());
    }
}
