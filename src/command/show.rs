use std::ffi::OsString;
use std::rc::Rc;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::func::expr::Expr;

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

        let model = context.as_model();

        if config.booleanized {
            for (uid, var) in model.variables() {
                let cpt = model.get_component_ref(var.component);
                let cpt = cpt.borrow();
                let e: Rc<Expr> = cpt.get_formula(var.value).convert_as();

                println!("{}: {},{} => {}", uid, cpt, var.value, e);
            }
        } else {
            println!("{}", model);
        }
        println!();

        CommandContext::Model(model)
    }
}
