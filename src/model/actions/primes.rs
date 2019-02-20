use crate::model;
use crate::model::io;
use clap::App;
use clap::SubCommand;
use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;


pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIPrimes{})
}

struct CLIPrimes;
impl CLIAction for CLIPrimes {
    fn name(&self) -> &'static str { "primes" }

    fn register_command(&self, mut app: App<'static, 'static>) -> App<'static, 'static> {
        app.subcommand(SubCommand::with_name(self.name())
            .about("Compute the prime implicants of the model's functions")
            .aliases(&["pi", "implicants"])
        )
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(PrimeBuilder::new(model))
    }
}


pub struct PrimeBuilder{
    model: LQModel,
}


impl PrimeBuilder {
    pub fn new(model: LQModel) -> PrimeBuilder {
        PrimeBuilder{model: model}
    }
}

impl ActionBuilder for PrimeBuilder {

    fn call(&self) {
        self.model.primes();
    }
}
