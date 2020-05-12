use crate::func::expr::Expr;
use crate::model::{QModel, LQModelRef};

use crate::command::{CLICommand, CommandContext};
use std::ffi::OsString;
use std::rc::Rc;
use std::sync::Arc;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "show";
static ABOUT: &str = "Display the current model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    #[structopt(short, long)]
    booleanized: bool,
}

struct CLIShow;

pub fn cli_action() -> Arc<dyn CLICommand> {
    Arc::new(CLIShow {})
}

impl CLICommand for CLIShow {
    fn name(&self) -> &'static str {
        NAME
    }
    fn about(&self) -> &'static str {
        ABOUT
    }

    fn clap(&self) -> App {
        Config::clap()
    }

    fn aliases(&self) -> &[&'static str] {
        &["display", "print"]
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = context.as_model();
        let config: Config = Config::from_iter(args);

        // Save the model
        if config.booleanized {
            for (uid, var) in model.variables() {
                let cpt = model.get_component_ref(var.component);
                let cpt = cpt.borrow();
                let e: Rc<Expr> = cpt.get_formula(var.value).convert_as();

                println!("{}: {},{} => {}", uid, cpt.name, var.value, e);
            }
        } else {
            println!("{}", model.for_display());
        }
        println!();

        CommandContext::Model(model)
    }
}
