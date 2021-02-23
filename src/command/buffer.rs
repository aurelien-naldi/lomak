use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::helper::error::EmptyLomakResult;
use crate::model::modifier::buffer::{BufferConfig, BufferingStrategy};
use std::ops::DerefMut;

static NAME: &str = "buffer";
static ABOUT: &str = "Add buffer components to delay interactions";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// The buffering strategy
    #[structopt(short, long)]
    delay: bool,
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
        let cli_config: Config = Config::from_iter(args);
        let smodel = context.get_model()?;
        let mut model = smodel.borrow_mut();

        let strategy = if cli_config.delay {
            BufferingStrategy::Delay
        } else {
            BufferingStrategy::AllBuffers
        };

        // TODO: more strategies and custon rules

        let mut config = BufferConfig::new(model.deref_mut(), strategy);
        config.apply();

        Ok(())
    }
}
