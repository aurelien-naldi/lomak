use crate::func::expr::Expr;
use crate::model::{QModel, LQModelRef};
use crate::solver;
use crate::solver::Solver;
use crate::command::{CLICommand, CommandContext};
use crate::solver::SolverMode;

use std::ffi::OsString;
use itertools::Itertools;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use clap::App;
use structopt::StructOpt;
use crate::model::actions::trapspaces::TrapspacesBuilder;

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

pub struct CLI;
impl CLICommand for CLI {

    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn aliases(&self) -> &[&'static str] {
        &["fixed-patterns"]
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = context.as_model();
        let config: Config = Config::from_iter(args);

        let mut builder = TrapspacesBuilder::new(model.as_ref());
        builder.set_percolate(config.percolate);

        if config.elementary {
            builder.show_elementary();
        }
        if config.all {
            builder.show_all();
        }

        builder.call();

        CommandContext::Model( model )
    }
}
