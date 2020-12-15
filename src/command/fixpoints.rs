use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::error::EmptyLomakResult;
use crate::model::actions::fixpoints::FixedBuilder;

static NAME: &str = "fixpoints";
static ABOUT: &str = "Compute the fixed points of the model";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// Maximal number of results
    #[structopt(short, long)]
    max: Option<usize>,

    /// Select output components
    #[structopt(short, long)]
    displayed: Option<Vec<String>>,

    /// Enforce additional constraints
    #[structopt(short, long)]
    enforce: Option<Vec<String>>,
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

    fn run(&self, context: &mut CommandContext, args: &[OsString]) -> EmptyLomakResult {
        let config: Config = Config::from_iter(args);

        // Create the fixpoint builder
        let smodel = context.get_model()?;
        let mut builder = FixedBuilder::new(smodel);

        // Apply extra restrictions if any
        if let Some(enforce) = config.enforce {
            for r in enforce {
                // enforce a constraint by restricting the opposite
                builder.restrict_by_name(&r, false);
            }
        }

        // Search the fixpoints and retrieve the results
        let mut result = builder.solve(config.max);

        // Select the listed variables and display the results
        if let Some(display) = config.displayed {
            result.set_displayed_names(Some(display));
        }
        println!("Fixed points:");
        println!("{}", result);
        println!("---");

        Ok(())
    }
}
