use std::collections::HashMap;

use itertools::Itertools;

use crate::func::expr::Expr;
use crate::model::actions::fixpoints::FixedPoints;
use crate::model::SharedModel;
use crate::solver;
use crate::solver::Solver;
use crate::solver::SolverMode;
use std::ops::Deref;

pub struct TrapspacesBuilder {
    model: SharedModel,
    filters: HashMap<usize, bool>,
    percolate: bool,
    mode: SolverMode,
}

impl TrapspacesBuilder {
    pub fn new(model: SharedModel) -> Self {
        TrapspacesBuilder {
            model,
            filters: HashMap::new(),
            percolate: false,
            mode: SolverMode::MAX,
        }
    }

    pub fn filter(&mut self, uid: usize, b: bool) {
        self.filters.insert(uid, b);
    }

    pub fn set_percolate(&mut self, b: bool) -> &mut Self {
        self.percolate = b;
        self
    }
    pub fn percolate(&mut self) -> &mut Self {
        self.set_percolate(true)
    }

    pub fn show_all(&mut self) -> &mut Self {
        self.mode = SolverMode::ALL;
        self
    }
    pub fn show_elementary(&mut self) -> &mut Self {
        self.mode = SolverMode::MIN;
        self
    }

    pub fn solve(&self, max: Option<usize>) -> FixedPoints {
        let mut solver = solver::get_solver(self.mode);

        let model = self.model.borrow();

        // Add all variables
        let s = model
            .variables
            .iter()
            .map(|vid| format!("v{}; v{}", 2 * vid, 2 * vid + 1))
            .join("; ");
        let s = format!("{{{}}}.\n", s);
        solver.add(&s);

        // A variable can only be fixed at a specific value
        for vid in &model.variables {
            solver.add(&format!(":- v{}, v{}.\n", 2 * vid, 2 * vid + 1));
        }

        for vid in model.variables() {
            let e = model.get_var_rule(*vid);
            let ne = e.not();
            restrict(&mut *solver, &e, 2 * vid + 1);
            restrict(&mut *solver, &ne, 2 * vid);

            if self.percolate {
                enforce(&mut *solver, &e, 2 * vid);
                enforce(&mut *solver, &ne, 2 * vid + 1);
            }
        }

        // Remove the full state space from the solutions when computing elementary trapspaces
        if self.mode == SolverMode::MIN {
            let s = model
                .variables()
                .iter()
                .map(|vid| format!("not v{}, not v{}", 2 * vid, 2 * vid + 1))
                .join(", ");
            let s = format!(":- {}.\n", s);
            solver.add(&s);
        }

        let mut results = solver.solve();
        results.set_halved();

        let patterns = results
            .into_iter()
            .map(|r| r.as_pattern())
            .take(max.unwrap_or(10000))
            .collect_vec();

        FixedPoints::new(model.deref(), patterns)
    }
}

fn restrict(solver: &mut dyn Solver, e: &Expr, u: usize) {
    for p in e.prime_implicants().iter() {
        let s = p
            .positive()
            .iter()
            .map(|r| format!("not v{}", 2 * r + 1))
            .chain(p.negative().iter().map(|r| format!("not v{}", 2 * r)))
            .join(",");

        if s.is_empty() {
            solver.add(&format!(":- v{}.\n", u));
        } else {
            solver.add(&format!(":- v{}, {}.\n", u, s));
        }
    }
}

fn enforce(solver: &mut dyn Solver, e: &Expr, u: usize) {
    for p in e.prime_implicants().iter() {
        let s = p
            .positive()
            .iter()
            .map(|r| format!("v{}", 2 * r))
            .chain(p.negative().iter().map(|r| format!("v{}", 2 * r + 1)))
            .join(",");

        if s.is_empty() {
            solver.add(&format!("v{}.\n", u));
        } else {
            solver.add(&format!("v{} :- {}.\n", u, s));
        }
    }
}
