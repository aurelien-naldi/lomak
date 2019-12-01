use crate::func::expr::Expr;
use crate::model::actions::ActionBuilder;
use crate::model::actions::ArgumentDescr;
use crate::model::actions::CLIAction;
use crate::model::QModel;

use crate::solver;
use crate::solver::Solver;

use crate::solver::clingo::ClingoProblem;
use crate::solver::SolverMode;
use crate::solver::SolverResults;
use itertools::Itertools;
use std::collections::HashMap;
use std::rc::Rc;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec! {
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
        ArgumentDescr::new("elementary")
            .help("Show only elementary trapspaces, i.e. minimal stable motifs")
            .long("elementary")
            .short("e"),
        ArgumentDescr::new("all")
            .help("All trapspaces instead of only the terminal ones")
            .long("all")
            .short("a"),
    };
}

impl dyn QModel {
    pub fn trapspaces(&'_ self) -> TrapspacesBuilder<'_> {
        TrapspacesBuilder::new(self)
    }
}

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed {})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str {
        "trapspaces"
    }
    fn about(&self) -> &'static str {
        "Compute the trapspaces (stable patterns) of the model"
    }

    fn arguments(&self) -> &'static [ArgumentDescr] {
        &PARAMETERS
    }

    fn builder<'a>(&self, model: &'a dyn QModel) -> Box<dyn ActionBuilder + 'a> {
        Box::new(TrapspacesBuilder::new(model))
    }
}

pub struct TrapspacesBuilder<'a> {
    model: &'a dyn QModel,
    filters: HashMap<usize, bool>,
    percolate: bool,
    mode: SolverMode,
}

impl<'a> TrapspacesBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> Self {
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
}

impl TrapspacesBuilder<'_> {
    pub fn percolate(&mut self) -> &mut Self {
        self.percolate = true;
        self
    }
    pub fn show_all(&mut self) -> &mut Self {
        self.mode = SolverMode::ALL;
        self
    }
    pub fn show_elementary(&mut self) -> &mut Self {
        self.mode = SolverMode::MIN;
        self
    }
}

impl ActionBuilder for TrapspacesBuilder<'_> {
    fn set_flag(&mut self, flag: &str) {
        match flag {
            "percolate" => self.percolate(),
            "all" => self.show_all(),
            "elementary" => self.show_elementary(),
            _ => self,
        };
    }

    fn call(&self) {
        let mut solver = solver::get_solver(self.mode);

        // Add all variables
        let s = self
            .model
            .variables()
            .map(|(uid, _)| format!("v{}; v{}", 2 * uid, 2 * uid + 1))
            .join("; ");
        let s = format!("{{{}}}.\n", s);
        solver.add(&s);

        // A variable can only be fixed at a specific value
        for (uid, _) in self.model.variables() {
            solver.add(&format!(":- v{}, v{}.\n", 2 * uid, 2 * uid + 1));
        }

        for (uid, var) in self.model.variables() {
            let e: Rc<Expr> = self.model.get_component(var.component).as_func(var.value);
            let ne = e.not();
            restrict(&mut solver, &e, 2 * uid + 1);
            restrict(&mut solver, &ne, 2 * uid);

            if self.percolate {
                enforce(&mut solver, &e, 2 * uid);
                enforce(&mut solver, &ne, 2 * uid + 1);
            }

            // Remove the full state space from the solutions when computing elementary trapspaces
            if self.mode == SolverMode::MIN {
                let s = self
                    .model
                    .variables()
                    .map(|(uid, _)| format!("not v{}, not v{}", 2 * uid, 2 * uid + 1))
                    .join(", ");
                let s = format!(":- {}.\n", s);
                solver.add(&s);
            }
        }

        let mut results = solver.solve_clingo();
        results.set_halved();
        for r in results {
            println!("{}", r);
        }
    }
}

fn restrict(solver: &mut ClingoProblem, e: &Expr, u: usize) {
    for p in e.prime_implicants().items() {
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

fn enforce(solver: &mut ClingoProblem, e: &Expr, u: usize) {
    for p in e.prime_implicants().items() {
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
