use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::sync::Arc;

use crate::model::SharedModel;

mod help;
mod load;

mod buffer;
mod perturbation;
mod rename;

mod fixpoints;
mod primes;
mod save;
mod show;
mod trapspaces;

lazy_static! {
    pub static ref COMMANDS: CommandManager = CommandManager::new()
        // Help: show commands
        .register( Arc::new( help::CLI{} ))

        // Load a new model
        .register( Arc::new( load::CLI{} ))

        // Model modifiers
        .register( Arc::new( perturbation::CLI{} ))
        .register( Arc::new( rename::CLI{} ))
        .register( Arc::new( buffer::CLI{} ))

        // Actions
        .register( Arc::new( show::CLI{} ))
        .register( Arc::new( save::CLI{} ))
        .register( Arc::new( primes::CLI{} ))
        .register( Arc::new( fixpoints::CLI{} ))
        .register( Arc::new( trapspaces::CLI{} ))
        ;
}

pub struct SelectedArgs {
    all_args: Vec<OsString>,
    next_slice: usize,
}

pub struct CommandManager {
    services: HashMap<&'static str, Arc<dyn CLICommand>>,
    aliases: HashMap<&'static str, &'static str>,
}

impl CommandManager {
    /// Init function to load all available actions
    pub fn new() -> CommandManager {
        CommandManager {
            services: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn register(mut self, action: Arc<dyn CLICommand>) -> Self {
        let name = action.name();
        for alias in action.aliases().iter() {
            self.aliases.insert(alias, name);
        }
        self.services.insert(name, action);
        self
    }

    fn unroll_alias<'a>(&'a self, name: &'a str) -> &'a str {
        self.aliases.get(name).unwrap_or(&name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.services.contains_key(self.unroll_alias(name))
    }

    pub fn get_command(&self, name: &str) -> Option<Arc<dyn CLICommand>> {
        self.services
            .get(self.unroll_alias(name))
            .map(|c| Arc::clone(c))
    }

    pub fn print_commands(&self) {
        println!("Available commands:");
        for srv in self.services.iter() {
            println!("  {:20} {}", srv.0, srv.1.about());
        }
    }
}

pub enum CommandContext {
    Empty,
    Model(SharedModel),
}

impl CommandContext {
    pub fn get_model(&self) -> SharedModel {
        match self {
            CommandContext::Model(m) => m.clone(),
            _ => panic!("No model in the context"),
        }
    }
}

pub trait CLICommand: Sync + Send {
    fn name(&self) -> &'static str;

    fn about(&self) -> &'static str;

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn run(&self, context: CommandContext, args: &[OsString]) -> CommandContext;
}

impl SelectedArgs {
    pub fn new() -> Self {
        let argsos = env::args_os();
        SelectedArgs {
            all_args: argsos.collect(),
            next_slice: 0,
        }
    }

    pub fn has_next(&self) -> bool {
        self.next_slice < self.all_args.len()
    }

    pub fn parse_next<'a>(&mut self, context: CommandContext) -> CommandContext {
        self.run_next_command(context, &COMMANDS)
    }

    fn run_next_command(
        &mut self,
        context: CommandContext,
        manager: &CommandManager,
    ) -> CommandContext {
        let next_command = self.all_args[self.next_slice].to_str().unwrap();
        let cmd = match manager.get_command(next_command) {
            None => return context,
            Some(c) => c,
        };

        let next_args = self.scan();

        cmd.run(context, next_args)
    }

    pub fn scan(&mut self) -> &[OsString] {
        let start = self.next_slice;

        // Find the end of the next slice!
        let mut end = self.all_args.len();
        for i in start + 1..end {
            let cur = self.all_args[i].to_str().unwrap();
            if COMMANDS.contains(cur) {
                end = i;
                break;
            }
        }

        self.next_slice = end;
        &self.all_args[start..end]
    }
}
