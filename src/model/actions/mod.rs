use crate::command::CommandManager;
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
