use crate::model;
use crate::model::io;
use clap::App;
use clap::SubCommand;
use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::func::expr::Expr;

use crate::solver::Solver;
use crate::solver;

use itertools::Itertools;


pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed{})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str { "fixpoints" }

    fn register_command(&self, mut app: App<'static, 'static>) -> App<'static, 'static> {
        app.subcommand(SubCommand::with_name(self.name())
            .about("Compute the fixed points of the model")
            .aliases(&["fixed", "stable"])
        )
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(FixedBuilder::new(model))
    }
}


pub struct FixedBuilder{
    model: LQModel,
}


impl FixedBuilder {
    pub fn new(model: LQModel) -> FixedBuilder {
        FixedBuilder{model: model}
    }
}

impl ActionBuilder for FixedBuilder {

    fn call(&self) {
        let mut solver = solver::get_solver();
        let rules = self.model.rules();

        let s = rules.keys()
            .map(|u|format!("v{}", u))
            .join("; ");
        let s = format!("{{{}}}.", s);
        solver.add( &s);

        for (u, f) in rules {
            let cur = Expr::ATOM(*u);
            let e = &f.as_expr();
            for p in cur.and(e).or(&cur.not().and(&e.not())).not().prime_implicants().items() {
                solver.restrict(p);
            }
        }

        solver.solve();
    }

}
