use crate::func::expr;
use crate::func::paths;
use crate::model::{LQModelRef, QModel};

use crate::command::CLICommand;
use std::rc::Rc;
use std::sync::Arc;
use structopt::StructOpt;
use crate::model::actions::CLIAction;

static NAME: &str = "primes";
static ABOUT: &str = "Compute the prime implicants of the model's functions";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct PrimeConfig {
    /// Output prime implicants as JSON
    #[structopt(short, long)]
    json: bool,
}

pub fn cli_action() -> Arc<dyn CLICommand> {
    Arc::new(CLIPrimes {})
}

struct CLIPrimes;
impl CLIAction for CLIPrimes {
    type Config = PrimeConfig;

    fn name(&self) -> &'static str {
        NAME
    }
    fn about(&self) -> &'static str {
        let t = PrimeConfig{json:false};
        ABOUT
    }
    fn aliases(&self) -> &[&'static str] {
        &["pi", "implicants"]
    }

    fn run_model(&self, model: &LQModelRef, config: Self::Config) {
        if config.json {
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
