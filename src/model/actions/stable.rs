use crate::func::expr::Expr;
use crate::model::{QModel, LQModelRef};
use crate::solver;
use crate::command::{CLICommand, CommandContext};
use crate::func::paths::LiteralSet;
use crate::solver::SolverMode;

use std::ffi::OsString;
use itertools::Itertools;
use std::rc::Rc;
use std::sync::Arc;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "fixedpoints";
static ABOUT: &str = "Compute the fixed points of the model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Select output components
    displayed: Vec<String>,
}

impl dyn QModel {
    pub fn fixpoints(&'_ self) -> FixedBuilder<'_> {
        FixedBuilder::new(self)
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

    fn clap(&self) -> App {
        Config::clap()
    }

    fn aliases(&self) -> &[&'static str] {
        &["fixed", "stable", "fp"]
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = context.as_model();
        let config: Config = Config::from_iter(args);

        let builder = FixedBuilder::new(model.as_ref()).config(&config);
        builder.call();

        CommandContext::Model( model )
    }

}

pub struct FixedBuilder<'a> {
    model: &'a dyn QModel,
    restriction: Option<LiteralSet>,
    displayed: Option<Vec<usize>>,
}

impl<'a> FixedBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> FixedBuilder<'a> {
        FixedBuilder {
            model,
            restriction: None,
            displayed: None,
        }
    }

    fn config(mut self, config: &Config) -> Self {
        self.set_displayed_names(config.displayed.iter().map(|s|s.as_str()).collect());
        self
    }

    pub fn set_displayed_names(&mut self, names: Vec<&str>) {
        if self.displayed.is_none() {
            self.displayed = Some(vec![]);
        }
        let displayed = self.displayed.as_mut().unwrap();
        for name in names {
            // FIXME: multi-valued case
            let uid = self.model.variable_by_name(name);
            match &uid {
                Some(uid) => displayed.push(*uid),
                None => println!("Selected display component not found: {}", name),
            }
        }
    }

    pub fn call(&self) {
        let mut solver = solver::get_solver(SolverMode::ALL);
        let s = self
            .model
            .variables()
            .map(|(uid, _)| format!("v{}", uid))
            .join("; ");
        let s = format!("{{{}}}.", s);
        solver.add(&s);

        for (uid, var) in self.model.variables() {
            let cpt = self.model.get_component_ref(var.component);
            let cpt = cpt.borrow();
            let cur = Expr::ATOM(uid);
            let e: Rc<Expr> = cpt.get_formula(var.value).convert_as();
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

        let results = solver.solve();
        match &self.displayed {
            None => {
                let s = self
                    .model
                    .variables()
                    .map(|(uid, _)| self.model.name(uid))
                    .join(" ");
                println!("{}", s);
            }
            Some(dsp) => {
                let s = dsp.iter().map(|uid| self.model.get_name(*uid)).join(" ");
                println!("{}", s);
            }
        }
        for r in results {
            println!("{}", r.filter(&self.displayed));
        }
    }
}
