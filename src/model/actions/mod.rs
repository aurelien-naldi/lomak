use crate::command::{CommandManager, CLICommand, CommandContext};
use structopt::StructOpt;
use clap::App;
use crate::model::LQModelRef;
use std::ffi::OsString;

pub mod export;
pub mod primes;
pub mod show;
pub mod stable;
pub mod trapspaces;

lazy_static! {
    pub static ref ACTIONS: CommandManager = CommandManager::new()
        .register(export::cli_action())
        .register(show::cli_action())
        .register(primes::cli_action())
        .register(stable::cli_action())
        .register(trapspaces::cli_action());
}

pub trait CLIAction: Sync + Send {
    type Config: StructOpt;

    fn name(&self) -> &'static str;

    fn about(&self) -> &'static str;

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn run_model(&self, model: &LQModelRef, config: Self::Config);
}


impl<T: CLIAction> CLICommand for T {

    fn name(&self) -> &'static str {
        CLIAction::name(self)
    }

    fn about(&self) -> &'static str {
        CLIAction::about(self)
    }

    fn help(&self) {
        T::Config::clap().print_help();
    }

    fn aliases(&self) -> &[&'static str] {
        CLIAction::aliases(self)
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let model = match &context {
            CommandContext::Model(m) => m,
            _ => panic!("invalid context"),
        };

        let config = T::Config::from_iter(args);
        self.run_model(model, config);

        // TODO: should it return the model or an empty context?
        context
    }
}
