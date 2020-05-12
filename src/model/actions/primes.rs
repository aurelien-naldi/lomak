use crate::func::expr;
use crate::func::paths;
use crate::model::{LQModelRef, QModel};
use crate::command::{CLICommand,CommandContext};

use std::ffi::OsString;
use std::rc::Rc;
use std::sync::Arc;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "primes";
static ABOUT: &str = "Compute the prime implicants of the model's functions";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Output prime implicants as JSON
    #[structopt(short, long)]
    json: bool,
}

pub fn cli_action() -> Arc<dyn CLICommand> {
    Arc::new(CLIPrimes {})
}

struct CLIPrimes;
impl CLICommand for CLIPrimes {
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
        &["pi", "implicants"]
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = context.as_model();
        let config: Config = Config::from_iter(args);

        config.show_primes(&model);

        CommandContext::Model( model )
    }
}

impl Config {
    fn show_primes(&self, model: &LQModelRef) {
        if self.json {
            json(model);
        } else {
            for (uid, var) in model.variables() {
                let primes: Rc<paths::Paths> = model
                    .get_component_ref(var.component)
                    .borrow()
                    .get_formula(var.value)
                    .convert_as();
                println!("PI {}:\n{}", model.name(uid), primes);
            }
        }
    }
}

pub fn json(model: &LQModelRef) {
    println!("{{");
    let mut first = true;
    let namer = model.as_namer();
    for (uid, var) in model.variables() {
        if first {
            first = false;
        } else {
            println!(",");
        }
        let cpt = model.get_component_ref(var.component);
        let cpt = cpt.borrow();
        // FIXME: should it be the name of the variable instead of the component?
        let name = &cpt.name;
        let pos_primes: Rc<paths::Paths> = cpt.get_formula(var.value).convert_as();
        let neg_primes = cpt
            .get_formula(var.value)
            .convert_as::<expr::Expr>()
            .not()
            .prime_implicants();
        println!("\"{}\":[", name);
        neg_primes.to_json(namer);
        println!(",");
        pos_primes.to_json(namer);
        print!("]");
    }
    println!("\n}}");
}
