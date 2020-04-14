use crate::command::{CLICommand, CommandContext};
use crate::model::LQModelRef;

use std::ffi::OsString;
use std::sync::Arc;
use structopt::StructOpt;

static NAME: &str = "rename";
static ABOUT: &str = "Rename one or several components";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct RenameConfig {
    /// The original name
    source: String,

    /// The target name
    target: String,
}

pub struct CLIRename;

pub fn cli_modifier() -> Arc<dyn CLICommand> {
    Arc::new(CLIRename {})
}

impl CLICommand for CLIRename {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn help(&self) {
        RenameConfig::clap().print_help();
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = match &mut context {
            CommandContext::Model(m) => m,
            _ => panic!("invalid context"),
        };

        let config: RenameConfig = RenameConfig::from_iter(args);

        // TODO: multiple rename actions ?
        model.rename(&config.source, config.target);

        context
    }
}
