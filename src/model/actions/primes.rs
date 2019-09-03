use crate::func::expr;
use crate::func::paths;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
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

    fn builder<'a>(&self, model: &'a dyn QModel) -> Box<dyn ActionBuilder + 'a> {
        Box::new(PrimeBuilder::new(model))
    }
}

pub struct PrimeBuilder<'a> {
    model: &'a dyn QModel,
}

impl<'a> PrimeBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> PrimeBuilder<'a> {
        PrimeBuilder { model: model }
    }
}

impl ActionBuilder for PrimeBuilder<'_> {
    fn call(&self) {
        println!("LOOKING FOR PRIMES!!");
        for uid in self.model.variables() {
            let func: expr::Expr = self.model.rule(*uid).as_func();
            let primes: paths::Paths = self.model.rule(*uid).as_func();
            println!("PI {}: {}", uid, primes);
        }
    }
}

impl<'a> PrimeBuilder<'a> {
    pub fn json(&self) {
        println!("{{");
        let mut first = true;
        let namer = self.model.as_namer();
        for uid in self.model.variables() {
            if first {
                first = false;
            } else {
                println!(",");
            }
            let rule = self.model.rule(*uid);
            let name = self.model.get_name(*uid);
            let pos_primes: paths::Paths = rule.as_func();
            let neg_primes = rule.as_func::<expr::Expr>().not().prime_implicants();
            println!("\"{}\":[", name);
            neg_primes.to_json(namer);
            println!(",");
            pos_primes.to_json(namer);
            print!("]");
        }
        println!("\n}}");
    }
}
