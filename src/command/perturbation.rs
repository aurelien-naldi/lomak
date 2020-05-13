use regex::Regex;

use crate::command::{CLICommand, CommandContext};
use crate::model::LQModelRef;

use std::borrow::BorrowMut;
use std::ffi::OsString;
use clap::App;
use structopt::StructOpt;

static NAME: &str = "perturbation";
static ABOUT: &str = "Apply a perturbation to one or several components";

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
}

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct Config {
    // Components to knock-out (fix level to 0)
    #[structopt(short, long)]
    ko: Vec<String>,

    /// perturbations: component%0
    perturbations: Vec<String>,
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
        // Start by parsing arguments to handle help without any context
        let config: Config = Config::from_iter(args);

        let mut model = context.as_model();
        for sid in &config.ko {
            if let Some(uid) = model.component_by_name(sid) {
                model.lock(uid, false);
            }
        }

        for arg in config.perturbations {
            match RE_PRT.captures(&arg) {
                None => println!("Invalid perturbation parameter: {}", arg),
                Some(cap) => {
                    if let Some(uid) = model.component_by_name(&cap[1]) {
                        match &cap[2] {
                            "0" => model.lock(uid, false),
                            "1" => model.lock(uid, true),
                            _ => println!("Invalid target value: {}", &cap[2]),
                        }
                    }
                }
            }
        }

        CommandContext::Model(model)
    }
}
