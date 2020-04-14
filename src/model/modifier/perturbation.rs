use regex::Regex;

use crate::command::{CLICommand, CommandContext};
use crate::model::LQModelRef;

use std::borrow::BorrowMut;
use std::ffi::OsString;
use std::sync::Arc;
use structopt::StructOpt;

static NAME: &str = "perturbation";
static ABOUT: &str = "Apply a perturbation to one or several components";

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
}

#[derive(Debug, StructOpt)]
#[structopt(name=NAME, about=ABOUT)]
struct PerturbationConfig {
    // Components to knock-out (fix level to 0)
    #[structopt(short, long)]
    ko: Vec<String>,

    /// perturbations: component%0
    perturbations: Vec<String>,
}

pub struct CLIPerturbation;

pub fn cli_modifier() -> Arc<dyn CLICommand> {
    Arc::new(CLIPerturbation {})
}

impl CLICommand for CLIPerturbation {
    fn name(&self) -> &'static str {
        NAME
    }

    fn about(&self) -> &'static str {
        ABOUT
    }

    fn help(&self) {
        PerturbationConfig::clap().print_help();
    }

    fn run(&self, mut context: CommandContext, args: &[OsString]) -> CommandContext {
        let mut model = match &mut context {
            CommandContext::Model(m) => m,
            _ => panic!("invalid context"),
        };

        let config: PerturbationConfig = PerturbationConfig::from_iter(args);

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

        context
    }
}
