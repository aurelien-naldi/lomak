use crate::func::expr;
use crate::func::paths;
use crate::model::actions::CLIAction;
use crate::model::actions::{ActionBuilder, ArgumentDescr};
use crate::model::QModel;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec! {
        ArgumentDescr::new("json")
            .help("Output prime implicants as JSON")
            .long("json")
            .short("j"),
    };
}

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

    fn arguments(&self) -> &'static [ArgumentDescr] {
        &PARAMETERS
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
    json: bool,
}

impl<'a> PrimeBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> PrimeBuilder<'a> {
        PrimeBuilder { model, json: false }
    }
}

impl ActionBuilder for PrimeBuilder<'_> {
    fn set_flag(&mut self, key: &str) {
        if let "json" = key {
            self.json = true;
        }
    }

    fn call(&self) {
        if self.json {
            self.json();
            return;
        }

        for (uid, _) in self.model.variables() {
            let primes: paths::Paths = self.model.rule(uid).as_func();
            println!("PI {}:\n{}", self.model.name(uid), primes);
        }
    }
}

impl<'a> PrimeBuilder<'a> {
    pub fn json(&self) {
        println!("{{");
        let mut first = true;
        let namer = self.model.as_namer();
        for (uid, _) in self.model.variables() {
            if first {
                first = false;
            } else {
                println!(",");
            }
            let rule = self.model.rule(uid);
            let name = self.model.get_name(uid);
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
