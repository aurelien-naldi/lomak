use crate::model::QModel;
use clap::{App, Arg, SubCommand};
use std::collections::HashMap;

pub mod export;
pub mod primes;
pub mod show;
pub mod stable;
pub mod trapspaces;

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
            services: HashMap::new(),
        }
        .register(export::cli_action())
        .register(primes::cli_action())
        .register(show::cli_action())
        .register(stable::cli_action())
        .register(trapspaces::cli_action())
    }

    fn register(mut self, action: Box<dyn CLIAction>) -> Self {
        self.services.insert(String::from(action.name()), action);
        self
    }
}

pub trait ActionBuilder {
    fn set_flag(&mut self, flag: &str) {
        eprintln!("This action has no flag ({})", flag);
    }

    fn set_value(&mut self, key: &str, value: &str) {
        eprintln!("This action has no value ({} = {})", key, value);
    }

    fn call(&self);
}

pub trait CLIAction: Sync {
    fn name(&self) -> &'static str;
    fn about(&self) -> &'static str;

    fn arguments(&self) -> &'static [ArgumentDescr] {
        &[]
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn register_command(&self, app: App<'static, 'static>) -> App<'static, 'static> {
        let mut cmd = SubCommand::with_name(self.name()).about(self.about());
        for alias in self.aliases() {
            cmd = cmd.alias(*alias);
        }
        for param in self.arguments() {
            cmd = cmd.arg(arg_from_descr(param));
        }

        app.subcommand(cmd)
    }

    fn builder<'a>(&self, model: &'a dyn QModel) -> Box<dyn ActionBuilder + 'a>;
}

pub fn arg_from_descr(param: &ArgumentDescr) -> Arg {
    let mut arg = Arg::with_name(&param.name)
        .help(&param.help)
        .required(param.required)
        .multiple(param.multiple)
        .takes_value(param.has_value);

    if param.long.is_some() {
        arg = arg.long(&param.long.as_ref().unwrap());
    }
    if param.short.is_some() {
        arg = arg.short(&param.short.as_ref().unwrap());
    }
    arg
}

pub struct ArgumentDescr {
    pub name: String,
    pub help: String,
    pub long: Option<String>,
    pub short: Option<String>,
    pub has_value: bool,
    pub multiple: bool,
    pub required: bool,
}

impl ArgumentDescr {
    pub fn new(name: &str) -> ArgumentDescr {
        ArgumentDescr {
            name: name.to_string(),
            help: String::from(""),
            long: None,
            short: None,
            has_value: false,
            multiple: false,
            required: false,
        }
    }

    pub fn help(mut self, help: &str) -> Self {
        self.help = help.to_string();
        self
    }
    pub fn long(mut self, long: &str) -> Self {
        self.long = Some(long.to_string());
        self
    }
    pub fn short(mut self, short: &str) -> Self {
        self.short = Some(short.to_string());
        self
    }
    pub fn has_value(mut self, b: bool) -> Self {
        self.has_value = b;
        self
    }
    pub fn multiple(mut self, b: bool) -> Self {
        self.multiple = b;
        self
    }
    pub fn required(mut self, b: bool) -> Self {
        self.required = b;
        self
    }
}

pub fn register_commands(mut app: App<'static, 'static>) -> App<'static, 'static> {
    for cli in ACTIONS.services.values() {
        app = cli.register_command(app)
    }
    app
}

pub fn run_command(cmd: &str, args: &clap::ArgMatches, model: &dyn QModel) {
    if let Some(cli) = ACTIONS.services.get(cmd) {
        let mut b = cli.builder(model);

        // Grab all cli arguments to feed the builder appropriately
        for descr in cli.arguments() {
            if descr.has_value {
                let value = args.value_of(&descr.name);
                if let Some(v) = value {
                    b.set_value(&descr.name, v);
                }
            // TODO: multiple matches?
            } else if args.is_present(&descr.name) {
                b.set_flag(&descr.name);
            }
        }

        b.call();
    } else {
    }
}
