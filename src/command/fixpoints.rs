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
use crate::model::actions::stable::FixedBuilder;

static NAME: &str = "fixpoints";
static ABOUT: &str = "Compute the fixed points of the model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Select output components
    displayed: Vec<String>,
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
        &["fixed", "stable", "fp"]
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let config: Config = Config::from_iter(args);

        let mut model = context.as_model();
        let mut builder = FixedBuilder::new(model.as_ref());

        builder.set_displayed_names(config.displayed.iter().map(|s|s.as_str()).collect());
        builder.call();

        CommandContext::Model( model )
    }

}
