use crate::func::expr;
use crate::func::paths;
use crate::func::VariableNamer;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::model::LQModelRef;
use crate::model::QModel;

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIPrimes {})
}

struct CLIPrimes;
impl CLIAction for CLIPrimes {
    fn name(&self) -> &'static str {
        "primes"
    }
    fn about(&self) -> &'static str {
        "Compute the prime implicants of the model's functions"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["pi", "implicants"]
    }

    fn builder(&self, model: LQModelRef) -> Box<dyn ActionBuilder> {
        Box::new(PrimeBuilder::new(model))
    }
}

pub struct PrimeBuilder {
    model: LQModelRef,
}

impl PrimeBuilder {
    pub fn new(model: LQModelRef) -> PrimeBuilder {
        PrimeBuilder { model: model }
    }
}

impl ActionBuilder for PrimeBuilder {
    fn call(&self) {
        for uid in self.model.variables() {
            let primes: paths::Paths = self.model.rule(*uid).as_func();
            println!("PI {}: {}", uid, primes);
        }
    }
}

impl PrimeBuilder {
    pub fn json(&self) {
        println!("{{");
        let mut first = true;
        for uid in self.model.variables() {
            if first {
                first = false;
            } else {
                println!(",");
            }
            let rule = self.model.rule(*uid);
            let name = self.model.name(*uid);
            let pos_primes: paths::Paths = rule.as_func();
            let neg_primes = rule.as_func::<expr::Expr>().not().prime_implicants();
            println!("\"{}\":[", name);
            neg_primes.to_json(&self.model);
            println!(",");
            pos_primes.to_json(&self.model);
            print!("]");
        }
        println!("\n}}");
    }
}
