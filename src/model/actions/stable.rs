use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::func::expr::Expr;

use crate::solver;

use itertools::Itertools;
use crate::solver::SolverMode;
use crate::func::paths::LiteralSet;


pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed{})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str { "fixpoints" }
    fn about(&self) -> &'static str { "Compute the fixed points of the model" }

    fn aliases(&self) -> &'static[&'static str] {
        &["fixed", "stable", "fp"]
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(FixedBuilder::new(model))
    }
}


pub struct FixedBuilder{
    model: LQModel,
    restriction: Option<LiteralSet>,
}


impl FixedBuilder {
    pub fn new(model: LQModel) -> FixedBuilder {
        FixedBuilder{model: model, restriction: None}
    }
}

impl ActionBuilder for FixedBuilder {

    fn call(&self) {
        let mut solver = solver::get_solver(SolverMode::ALL);
        let rules = self.model.rules();

        let s = rules.keys()
            .map(|u|format!("v{}", u))
            .join("; ");
        let s = format!("{{{}}}.", s);
        solver.add( &s);

        for (u, f) in rules {
            let cur = Expr::ATOM(*u);
            let e: Expr = f.as_func();
            for p in cur.not().and(&e).prime_implicants().items() {
                solver.restrict(p);
            }
            for p in cur.and(&e.not()).prime_implicants().items() {
                solver.restrict(p);
            }

            if self.restriction.is_some() {
                solver.restrict( self.restriction.as_ref().unwrap());
            }
        }

        solver.solve();
    }

}
