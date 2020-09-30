//! Define commands for the Command Line Interface
//!
//! This module provides facilities to register a collection of commands.
//! Each command is a wrapper over core API features defined in a private submodule.
//!
//! As the CLI enables to execute a chain of commands, the defined commands are recognized in the
//! list of parameters to extract separate lists of arguments for each successive command.


use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::sync::Arc;

use crate::model::SharedModel;

pub mod help;
pub mod load;

pub mod buffer;
pub mod perturbation;
pub mod rename;

pub mod fixpoints;
pub mod primes;
pub mod save;
pub mod show;
pub mod trapspaces;

lazy_static! {
    /// Single-instance CommandManager created and filled at runtime
    static ref COMMANDS: CommandManager = CommandManager::new()
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

/// Split the list of CLI parameters into separate slices for each successive command.
///
/// Scan the list of parameters to search for known commands and will consider that
/// they denote the start of the next command.
pub struct SelectedArgs {
    all_args: Vec<OsString>,
    next_slice: usize,
}

/// Register and retrieve commands
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

/// API for individual commands
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
