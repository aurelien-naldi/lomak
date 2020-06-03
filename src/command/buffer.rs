use std::ffi::OsString;

use structopt::StructOpt;

use crate::command::{CLICommand, CommandContext};
use crate::model::modifier::buffer::{BufferConfig, BufferingStrategy};
use crate::model::QModel;

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

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext {
        let _config: Config = Config::from_iter(args);

        // TODO: call the buffering tool

        context
    }
}

impl CLI {
    fn modify(&self, model: &mut QModel, parameters: &[&str]) {
        let strategy = match parameters {
            ["buffer"] => BufferingStrategy::ALLBUFFERS,
            ["delay"] => BufferingStrategy::DELAY,
            ["separate"] => BufferingStrategy::SEPARATING,
            _ => BufferingStrategy::CUSTOM,
        };
        let mut config = BufferConfig::new(model, strategy);

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
    }
}
