use std::collections::HashMap;
use clap::{App, Arg, SubCommand};
use crate::model::LQModel;


pub mod show;
pub mod export;
pub mod primes;
pub mod stable;

lazy_static! {
    pub static ref ACTIONS: ActionManager = ActionManager::new();
}

pub struct ActionManager {
    pub services: HashMap<String, Box<dyn CLIAction>>,
}

impl ActionManager {
    /// Init function to load all available actions
    fn new() -> ActionManager {
        ActionManager {
            services: HashMap::new()
        }
            .register(show::cli_action())
            .register(export::cli_action())
            .register(primes::cli_action())
            .register(stable::cli_action())
    }

    fn register(mut self, action: Box<dyn CLIAction>) -> Self {
        self.services.insert(String::from(action.name()), action);
        self
    }
}


pub trait ActionBuilder {
    fn set_flag(&mut self, _flag: &str) {}
    fn set_value(&mut self, _key: &str, _value: &str) {}

    fn set_args(&mut self, args: &clap::ArgMatches) {

    }

    fn call(&self);
}

pub trait CLIAction: Sync {
    fn name(&self) -> &'static str;
    fn about(&self) -> &'static str;

    fn arguments(&self) -> &'static[ArgumentDescr] {
        &[]
    }

    fn aliases(&self) -> &'static[&'static str] {
        &[]
    }

    fn register_command(&self, app: App<'static, 'static>) -> App<'static, 'static> {

        let mut cmd = SubCommand::with_name(self.name())
            .about(self.about());
        for alias in self.aliases() {
            cmd = cmd.alias(*alias);
        }
        for param in self.arguments() {
            let mut arg = Arg::with_name(&param.name)
                .help(&param.help)
                .required(param.required)
                .takes_value(param.has_value);

            if param.long.is_some() {
                arg = arg.long(&param.long.as_ref().unwrap());
            }
            if param.short.is_some() {
                arg = arg.short(&param.short.as_ref().unwrap());
            }
            cmd = cmd.arg(arg);
        }

        app.subcommand(cmd)
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder>;
}

pub struct ArgumentDescr {
    pub name: String,
    pub help: String,
    pub long: Option<String>,
    pub short: Option<String>,
    pub has_value: bool,
    pub required: bool,
}

impl ArgumentDescr {
    pub fn new(name: &str) -> ArgumentDescr {
        ArgumentDescr{
            name: name.to_string(),
            help: String::from(""),
            long: None,
            short: None,
            has_value: false,
            required: false,

        }
    }

    pub fn help(mut self, help:&str) -> Self {
        self.help = help.to_string();
        self
    }
    pub fn long(mut self, long:&str) -> Self {
        self.long = Some(long.to_string());
        self
    }
    pub fn short(mut self, short:&str) -> Self {
        self.short = Some(short.to_string());
        self
    }
    pub fn has_value(mut self, b:bool) -> Self {
        self.has_value = b;
        self
    }
    pub fn required(mut self, b:bool) -> Self {
        self.required = b;
        self
    }
}



pub fn register_commands(mut app: App<'static,'static>) -> App<'static,'static> {
    for cli in ACTIONS.services.values() {
        app = cli.register_command(app)
    }
    app
}


pub fn run_command(cmd: &str, args: &clap::ArgMatches, model: LQModel) {

    if let Some(cli) = ACTIONS.services.get(cmd) {
        let mut b = cli.builder(model);
        b.set_args(args);
        b.call();
    } else {

    }

}
