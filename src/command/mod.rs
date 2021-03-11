//! Define commands for the Command Line Interface
//!
//! This module provides facilities to register a collection of commands, where ach command,
//! defined in a private submodule, is a thin wrapper over core API features.
//!
//! The CLI enables to chain several commands, each can use and modify a global context, currently
//!limited to a shared model. The global CLI will thus start by searching command names in the full
//! list of arguments. The arguments between two successive commands define the arguments of the
//! first command.
//!
//! # Example
//!
//! The following command:
//!
//! ```lomak load -f sbml model.xml perturbation --ko Cmp3 fixpoints```
//!
//! defines the following subcommands:
//! * **load** ```-f sbml model.xml```
//! * **perturbation** ```--ko Cmp3```
//! * **fixpoints**

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::sync::Arc;

use crate::helper::error::{EmptyLomakResult, LomakError, LomakResult};
use crate::model::SharedModel;

// Use a macro to load all command modules and add them to the list of available commands
macro_rules! cmdmods {
    ( $( $x:ident ),* ) => {
        $( mod $x; )*
        /// Single-instance CommandManager created and filled at runtime
        static COMMANDS: Lazy<CommandManager> = Lazy::new(|| {
            CommandManager::default()
            $(  .register( Arc::new( $x::CLI{}))  )*
        });
    };
}

// Define all available commands
cmdmods!(
    help,
    load,
    buffer,
    perturbation,
    rename,
    fixpoints,
    reach,
    trapspaces,
    primes,
    save,
    show
);

pub fn help_cmd(context: &mut CommandContext) -> EmptyLomakResult {
    let cmd = COMMANDS.get_command("help").unwrap();
    cmd.run(context, &[])
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
#[derive(Default)]
pub struct CommandManager {
    services: HashMap<&'static str, Arc<dyn CLICommand>>,
    aliases: HashMap<&'static str, &'static str>,
}

impl CommandManager {
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
        println!("Available commands");
        println!("==================");
        for srv in self.services.iter() {
            println!("  {:20} {}", srv.0, srv.1.about());
        }
    }
}

/// The execution context to allow successive commands to share a model or additional data
#[derive(Default)]
pub struct CommandContext {
    models: HashMap<String, SharedModel>,
}

impl CommandContext {
    pub fn get_model(&self) -> LomakResult<SharedModel> {
        self.get_named_model(None)
    }

    pub fn get_named_model(&self, name: Option<&str>) -> LomakResult<SharedModel> {
        match self.models.get(name.unwrap_or("")) {
            Some(m) => Ok(m.clone()),
            None => Err(LomakError::MissingModel()),
        }
    }

    pub fn set_model(&mut self, model: SharedModel, name: Option<&str>) {
        self.models.insert(name.unwrap_or("").to_owned(), model);
    }
}

/// API for individual commands
pub trait CLICommand: Sync + Send {
    fn name(&self) -> &'static str;

    fn about(&self) -> &'static str;

    fn aliases(&self) -> &[&'static str] {
        &[]
    }

    fn run(&self, context: &mut CommandContext, args: &[OsString]) -> EmptyLomakResult;
}

impl SelectedArgs {
    pub fn new() -> Self {
        let argsos = env::args_os().collect();
        SelectedArgs {
            all_args: argsos,
            next_slice: 0,
        }
    }

    pub fn has_next(&self) -> bool {
        self.next_slice < self.all_args.len()
    }

    pub fn parse_next(&mut self, context: &mut CommandContext) -> EmptyLomakResult {
        self.run_next_command(context, &COMMANDS)
    }

    fn run_next_command(
        &mut self,
        context: &mut CommandContext,
        manager: &CommandManager,
    ) -> EmptyLomakResult {
        let next_command = self.all_args[self.next_slice].to_str().unwrap();
        let cmd = match manager.get_command(next_command) {
            None => return Ok(()),
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

impl Default for SelectedArgs {
    fn default() -> Self {
        Self::new()
    }
}
