use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::func::state::State;
use crate::helper::error::{EmptyLomakResult, GenericError};
use crate::model::actions::reach;
use crate::model::QModel;
use crate::variables::GroupedVariables;
use std::ops::Deref;

static NAME: &str = "reach";
static ABOUT: &str = "Reachability analysis";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Active components in the initial state
    #[structopt(short, long)]
    initial: Option<Vec<String>>,

    /// Active components in the target state
    #[structopt(short, long)]
    target: Option<Vec<String>>,
}

pub struct CLI;
impl CLICommand for CLI {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn run(&self, context: &mut CommandContext, args: &[OsString]) -> EmptyLomakResult {
        let config: Config = Config::from_iter(args);
        let smodel = context.get_model()?;
        let model = smodel.borrow();

        let init = state_from_cli(model.deref(), config.initial)?;
        let target = state_from_cli(model.deref(), config.target)?;

        if reach::most_permissive_reach(model.deref(), init, target) {
            println!("Reachable!");
        } else {
            println!("NOT Reachable!");
        }
        Ok(())
    }
}

fn state_from_cli(model: &QModel, names: Option<Vec<String>>) -> Result<State, GenericError> {
    let mut state = State::new();
    if let Some(names) = names {
        for name in &names {
            state.insert(model.get_handle_res(name)?);
        }
    }
    Ok(state)
}
