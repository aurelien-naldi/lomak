use crate::func::expr::Expr;
use crate::model::QModel;

use crate::solver;
use crate::solver::Solver;

use crate::command::{CLICommand, CommandContext};
use crate::solver::SolverMode;
use itertools::Itertools;
use std::collections::HashMap;
use std::ffi::OsString;
use std::rc::Rc;
use std::sync::Arc;
use structopt::StructOpt;

static NAME: &str = "trapspaces";
static ABOUT: &str = "Compute the trapspaces (stable patterns) of the model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Filter the results
    #[structopt(short, long)]
    filter: Option<Vec<String>>,

    /// Percolate (propagate) fixed components
    #[structopt(short, long)]
    percolate: bool,

    /// Show only elementary trapspaces, i.e. minimal stable motifs
    #[structopt(short, long)]
    elementary: bool,

    /// All trapspaces instead of only the terminal ones
    #[structopt(short, long)]
    all: bool,
}

impl dyn QModel {
    pub fn trapspaces(&'_ self) -> TrapspacesBuilder<'_> {
        TrapspacesBuilder::new(self)
    }
}

pub fn cli_action() -> Arc<dyn CLICommand> {
    Arc::new(CLIFixed {})
}

struct CLIFixed;
impl CLICommand for CLIFixed {
    fn name(&self) -> &'static str {
        NAME
    }
    fn about(&self) -> &'static str {
        ABOUT
    }

    fn help(&self) {
        Config::clap().print_help();
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let model = match &context {
            CommandContext::Model(m) => m,
            _ => panic!("invalid context"),
        };

        let config: Config = Config::from_iter(args);

        let builder = TrapspacesBuilder::new(model.as_ref());
        // FIXME: configure it
        builder.call();

        // TODO: should it return the model or an empty context?
        context
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
            let cpt = self.model.get_component_ref(var.component);
            let e: Rc<Expr> = cpt.borrow().get_formula(var.value).convert_as();
            let ne = e.not();
            restrict(&mut *solver, &e, 2 * uid + 1);
            restrict(&mut *solver, &ne, 2 * uid);

            if self.percolate {
                enforce(&mut *solver, &e, 2 * uid);
                enforce(&mut *solver, &ne, 2 * uid + 1);
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

        let mut results = solver.solve();
        results.set_halved();
        for r in results {
            println!("{}", r);
        }
    }
}

fn restrict(solver: &mut dyn Solver, e: &Expr, u: usize) {
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

fn enforce(solver: &mut dyn Solver, e: &Expr, u: usize) {
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
