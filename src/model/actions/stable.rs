use crate::func::expr::Expr;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::model::{LQModelRef, QModel};

use crate::solver;

use crate::func::paths::LiteralSet;
use crate::solver::SolverMode;
use itertools::Itertools;

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed {})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str {
        "fixpoints"
    }
    fn about(&self) -> &'static str {
        "Compute the fixed points of the model"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["fixed", "stable", "fp"]
    }

    fn builder(&self, model: LQModelRef) -> Box<dyn ActionBuilder> {
        Box::new(FixedBuilder::new(model))
    }
}

pub struct FixedBuilder {
    model: LQModelRef,
    restriction: Option<LiteralSet>,
}

impl FixedBuilder {
    pub fn new(model: LQModelRef) -> FixedBuilder {
        FixedBuilder {
            model: model,
            restriction: None,
        }
    }
}

impl ActionBuilder for FixedBuilder {
    fn call(&self) {
        let mut solver = solver::get_solver(SolverMode::ALL);
        let s = self.model.variables().iter().map(|uid| format!("v{}", uid)).join("; ");
        let s = format!("{{{}}}.", s);
        solver.add(&s);

        for uid in self.model.variables() {
            let rule = self.model.rule(*uid);
            let cur = Expr::ATOM(*uid);
            let e: Expr = rule.as_func();
            for p in cur.not().and(&e).prime_implicants().items() {
                solver.restrict(p);
            }
            for p in cur.and(&e.not()).prime_implicants().items() {
                solver.restrict(p);
            }

            if self.restriction.is_some() {
                solver.restrict(self.restriction.as_ref().unwrap());
            }
        }

        solver.solve();
    }
}
