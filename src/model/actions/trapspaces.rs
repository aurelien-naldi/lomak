use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::func::expr::Expr;

use crate::solver;

use itertools::Itertools;
use crate::solver::clingo::ClingoProblem;


pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed{})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str { "trapspaces" }
    fn about(&self) -> &'static str { "Compute the trapspaces (stable patterns) of the model" }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(TrapspacesBuilder::new(model))
    }
}


pub struct TrapspacesBuilder{
    model: LQModel,
}


impl TrapspacesBuilder {
    pub fn new(model: LQModel) -> Self {
        TrapspacesBuilder{model: model}
    }
}

impl ActionBuilder for TrapspacesBuilder {

    fn call(&self) {
        let mut solver = solver::get_solver();
        let rules = self.model.rules();

        // Add all variables
        let s = rules.keys()
            .map(|u|format!("v{}; v{}", 2*u, 2*u+1))
            .join("; ");
        let s = format!("{{{}}}.\n", s);
        solver.add( &s);

        // A variable can only be fixed at a specific value
        for u in rules.keys() {
            solver.add(&format!(":- v{}, v{}.\n", 2*u, 2*u+1));
        }

        for (u, f) in rules {
            let e = &f.as_expr();
            restrict(&mut solver, e, 2*u+1);
            restrict(&mut solver, &e.not(), 2*u);
        }

        solver.solve();
    }

}

fn restrict(solver: &mut ClingoProblem, e: &Expr, u: usize) {
    for p in e.prime_implicants().items() {
        let s = p.positive().iter().map(|r|format!("not v{}", 2*r+1))
            .chain(p.negative().iter().map(|r|format!("not v{}", 2*r)))
            .join(",");

        if s.len() > 0 {
            solver.add(&format!(":- v{}, {}.\n", u, s));
        } else {
            solver.add(&format!(":- v{}.\n", u));
        }
    }
}
