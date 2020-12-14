use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::model::{GroupedVariables, QModel};
use std::ops::Deref;
use crate::error::EmptyLomakResult;

static NAME: &str = "primes";
static ABOUT: &str = "Compute the prime implicants of the model's functions";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Output prime implicants as JSON
    #[structopt(short, long)]
    json: bool,
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
        &["pi", "implicants"]
    }

    fn run(&self, context: &mut CommandContext, args: &[OsString]) -> EmptyLomakResult {
        let config: Config = Config::from_iter(args);

        let smodel = context.get_model()?;
        let model = smodel.borrow();

        config.show_primes(model.deref());

        Ok(())
    }
}

impl Config {
    fn show_primes(&self, model: &QModel) {
        if self.json {
            json(model);
        } else {
            for vid in model.variables() {
                let primes = model.get_var_rule(*vid).prime_implicants();
                println!("PI {}:\n{}", model.get_name(*vid), primes);
            }
        }
    }
}

pub fn json(model: &QModel) {
    println!("{{");
    let mut first = true;
    for vid in model.variables() {
        if first {
            first = false;
        } else {
            println!(",");
        }
        let name = model.get_name(*vid);
        let rule = model.get_var_rule(*vid);
        let pos_primes = rule.prime_implicants();
        let neg_primes = rule.not().prime_implicants();
        println!("\"{}\":[", name);
        neg_primes.to_json(model);
        println!(",");
        pos_primes.to_json(model);
        print!("]");
    }
    println!("\n}}");
}
