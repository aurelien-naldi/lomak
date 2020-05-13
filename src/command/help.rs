use crate::command;
use std::ffi::OsString;
use std::rc::Rc;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "help";
static ABOUT: &str = "List available commands";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
}

pub struct CLI;
impl command::CLICommand for CLI {
    fn name(&self) -> &'static str {
        NAME
    }
    fn about(&self) -> &'static str {
        ABOUT
    }

    fn aliases(&self) -> &[&'static str] {
        &["display", "print"]
    }

    fn run(&self, mut context: command::CommandContext, args: &[OsString]) -> command::CommandContext {
        command::COMMANDS.print_commands();
        context
    }
}
