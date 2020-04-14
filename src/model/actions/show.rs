use crate::func::expr::Expr;
use crate::model::QModel;

use crate::command::{CLICommand, CommandContext};
use clap::App;
use std::ffi::OsString;
use std::rc::Rc;
use std::sync::Arc;
use structopt::StructOpt;

static NAME: &str = "show";
static ABOUT: &str = "Display the current model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct ShowConfig {
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

    fn help(&self) {
        ShowConfig::clap().print_help();
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["display", "print"]
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let model = match &context {
            CommandContext::Model(m) => m,
            _ => panic!("invalid context"),
        };

        let config: ShowConfig = ShowConfig::from_iter(args);

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

        // TODO: should it return the model or an empty context?
        context
    }
}
