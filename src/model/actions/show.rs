use crate::func::expr::Expr;
use crate::model::{QModel, LQModelRef};

use crate::command::CLICommand;
use std::rc::Rc;
use std::sync::Arc;
use structopt::StructOpt;
use crate::model::actions::CLIAction;

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

impl CLIAction for CLIShow {
    type Config = ShowConfig;

    fn name(&self) -> &'static str {
        NAME
    }
    fn about(&self) -> &'static str {
        ABOUT
    }

    fn aliases(&self) -> &[&'static str] {
        &["display", "print"]
    }

    fn run_model(&self, model: &LQModelRef, config: ShowConfig) {
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
    }
}
