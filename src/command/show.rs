use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};

static NAME: &str = "show";
static ABOUT: &str = "Display the current model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    #[structopt(short, long)]
    booleanized: bool,
}

pub struct CLI;
impl CLICommand for CLI {
    fn name(&self) -> &'static str {
        NAME
    }
    fn about(&self) -> &'static str {
        ABOUT
    }

    fn aliases(&self) -> &[&'static str] {
        &["display", "print"]
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let config: Config = Config::from_iter(args);

        let smodel = context.get_model();
        let model = smodel.borrow();

        if config.booleanized {
            for vid in model.variables() {
                let e = model.get_var_rule(*vid);
                println!("{} => {}", vid, e);
            }
        } else {
            println!("{}", model);
        }
        println!();

        context
    }
}
