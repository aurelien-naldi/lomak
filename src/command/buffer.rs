use crate::model::{QModel, LQModelRef};

use crate::command::{CLICommand, CommandContext};
use std::ffi::OsString;
use std::rc::Rc;
use clap::App;
use structopt::StructOpt;
use crate::model::modifier::buffer::{BufferingStrategy, BufferConfig};


static NAME: &str = "buffer";
static ABOUT: &str = "TODO: Add buffer components to delay interactions";

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    /// The buffering strategy
    strategy: String,
}

pub struct CLI;

impl CLICommand for CLI {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {

        let config: Config = Config::from_iter(args);

        // TODO: call the buffering tool

        context
    }
}

impl CLI {
    fn modify(&self, mut model: LQModelRef, parameters: &[&str]) -> LQModelRef {
        let strategy = match parameters {
            ["buffer"] => BufferingStrategy::ALLBUFFERS,
            ["delay"] => BufferingStrategy::DELAY,
            ["separate"] => BufferingStrategy::SEPARATING,
            _ => BufferingStrategy::CUSTOM,
        };
        let mut config = BufferConfig::new(model.as_mut(), strategy);

        if strategy == BufferingStrategy::CUSTOM {
            for arg in parameters {
                let split: Vec<&str> = arg.split(':').collect();
                if split.len() != 2 {
                    println!("invalid buffering pattern");
                    continue;
                }

                if split[1] == "*" {
                    config.add_delay_by_name(split[0]);
                    continue;
                }

                // TODO: handle multiple targets
                config.add_single_buffer_by_name(split[0], split[1]);
            }
        }

        model
    }
}
