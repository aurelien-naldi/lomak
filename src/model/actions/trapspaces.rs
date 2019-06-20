use crate::model::LQModel;
use crate::model::actions::ActionBuilder;
use crate::model::actions::CLIAction;
use crate::func::expr::Expr;
use crate::model::actions::ArgumentDescr;

use crate::solver;

use itertools::Itertools;
use crate::solver::clingo::ClingoProblem;
use std::collections::HashMap;
use crate::solver::SolverMode;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec!{
        ArgumentDescr::new("filter")
            .help("Filter the results")
            .long("filter")
            .short("f")
            .multiple(true)
            .has_value(true),
        ArgumentDescr::new("percolate")
            .help("Percolate (propagate) fixed components")
            .long("percolate")
            .short("p"),
        ArgumentDescr::new("terminal")
            .help("Only terminal trapspaces (lower bound for attractors)")
            .long("terminal")
            .short("t"),
    };
}

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed{})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str { "trapspaces" }
    fn about(&self) -> &'static str { "Compute the trapspaces (stable patterns) of the model" }

    fn arguments(&self) -> &'static[ArgumentDescr] {
        &PARAMETERS
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(TrapspacesBuilder::new(model))
    }
}


pub struct TrapspacesBuilder{
    model: LQModel,
    filters: HashMap<usize,bool>,
    percolate: bool,
    terminal: bool,
}


impl TrapspacesBuilder {
    pub fn new(model: LQModel) -> Self {
        TrapspacesBuilder{
            model: model,
            filters: HashMap::new(),
            percolate: false,
            terminal: false,
        }
    }

    pub fn filter(&mut self, uid: usize, b: bool) {
        self.filters.insert(uid, b);
    }
}

impl ActionBuilder for TrapspacesBuilder {

    fn set_flag(&mut self, flag: &str) {
        match flag {
            "percolate" => self.percolate = true,
            "terminal" => self.terminal = true,
            _ => (),
        }
    }

    fn call(&self) {
        let mode = match self.terminal {
            true => SolverMode::MAX,
            false => SolverMode::ALL,
        };
        let mut solver = solver::get_solver(mode);
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
            let e: Expr = f.as_func();
            let ne = e.not();
            restrict(&mut solver, &e, 2*u+1);
            restrict(&mut solver, &ne, 2*u);

            if self.percolate {
                enforce(&mut solver, &e, 2*u);
                enforce(&mut solver, &ne, 2*u+1);
            }
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

fn enforce(solver: &mut ClingoProblem, e: &Expr, u: usize) {
    for p in e.prime_implicants().items() {
        let s = p.positive().iter().map(|r|format!("v{}", 2*r))
            .chain(p.negative().iter().map(|r|format!("v{}", 2*r+1)))
            .join(",");

        if s.len() > 0 {
            solver.add(&format!("v{} :- {}.\n", u, s));
        } else {
            solver.add(&format!("v{}.\n", u));
        }
    }
}
