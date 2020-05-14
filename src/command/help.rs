use std::ffi::OsString;

use structopt::StructOpt;

use crate::command;

static NAME: &str = "help";
static ABOUT: &str = "List available commands";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {}

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

    fn run(&self, context: command::CommandContext, args: &[OsString]) -> command::CommandContext {
        let _config: Config = Config::from_iter(args);

        command::COMMANDS.print_commands();
        context
    }
}
