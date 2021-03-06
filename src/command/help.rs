use std::ffi::OsString;

use structopt::StructOpt;

use crate::command;
use crate::helper::error::EmptyLomakResult;
use crate::model::io;

static NAME: &str = "help";
static ABOUT: &str = "List available commands and formats";

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

    fn run(&self, _context: &mut command::CommandContext, args: &[OsString]) -> EmptyLomakResult {
        let _config: Config = Config::from_iter(args);

        command::COMMANDS.print_commands();
        println!();
        io::print_formats();
        Ok(())
    }
}
