use clap::App;
use clap::SubCommand;
use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::func::variables::VariableNamer;


pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIPrimes{})
}

struct CLIPrimes;
impl CLIAction for CLIPrimes {
    fn name(&self) -> &'static str { "primes" }

    fn register_command(&self, app: App<'static, 'static>) -> App<'static, 'static> {
        app.subcommand(SubCommand::with_name(self.name())
            .about("Compute the prime implicants of the model's functions")
            .aliases(&["pi", "implicants"])
        )
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(PrimeBuilder::new(model))
    }
}


pub struct PrimeBuilder {
    model: LQModel,
}


impl PrimeBuilder {
    pub fn new(model: LQModel) -> PrimeBuilder {
        PrimeBuilder{model: model}
    }
}

impl ActionBuilder for PrimeBuilder {
    fn call(&self) {
        for (u, f) in self.model.rules() {
            let primes = f.as_expr().prime_implicants();
            println!("PI {}: {}", u, primes);
        }
    }
}

impl PrimeBuilder {

    pub fn json(&self) {
        println!("{{");
        let mut first = true;
        for (u, f) in self.model.rules() {
            if first {
                first = false;
            } else {
                println!(",");
            }
            let name = self.model.get_name(*u);
            let pos_primes = f.as_expr().prime_implicants();
            let neg_primes = f.as_expr().not().prime_implicants();
            println!("\"{}\":[", name);
            neg_primes.to_json(&self.model);
            println!(",");
            pos_primes.to_json(&self.model);
            print!("]");
        }
        println!("\n}}");
    }

}
