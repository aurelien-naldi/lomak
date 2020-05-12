use clap::{App, AppSettings, Arg, ArgMatches};

use std::env;
use std::env::ArgsOs;
use std::ffi::OsString;

use crate::model::actions;
use crate::model::io;
use crate::model::modifier;
use crate::model::LQModelRef;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SelectedArgs {
    all_args: Vec<OsString>,
    next_slice: usize,
    next_type: CommandType,
}

enum CommandType {
    Start,
    Modifier,
    Action,
    Done,
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

    pub fn register(mut self, action: Arc<CLICommand>) -> Self {
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
}

pub enum CommandContext {
    Empty,
    Model(LQModelRef),
}

impl CommandContext {

    pub fn as_model(self) -> LQModelRef {
        match (self) {
            CommandContext::Model(m) => m,
            _ => panic!("No model in the context"),
        }
    }
}

pub trait CLICommand: Sync + Send {
    fn name(&self) -> &'static str;

    fn about(&self) -> &'static str;

    fn clap(&self) -> App;

    fn help(&self) {
        self.clap().print_help();
    }

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
            next_type: CommandType::Start,
        }
    }

    pub fn has_next(&self) -> bool {
        self.next_slice < self.all_args.len()
    }

    pub fn parse_next<'a>(&mut self, context: CommandContext) -> CommandContext {
        match self.next_type {
            CommandType::Start => self.run_next_command(context, &io::LOADERS),
            CommandType::Modifier => self.run_next_command(context, &modifier::MODIFIERS),
            CommandType::Action => self.run_next_command(context, &actions::ACTIONS),
            CommandType::Done => context,
        }
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
        self.next_type = CommandType::Done;
        for i in start + 1..end {
            let cur = self.all_args[i].to_str().unwrap();
            if io::LOADERS.contains(cur) {
                self.next_type = CommandType::Start;
                end = i;
                break;
            }
            if modifier::MODIFIERS.contains(cur) {
                self.next_type = CommandType::Modifier;
                end = i;
                break;
            }
            if actions::ACTIONS.contains(cur) {
                self.next_type = CommandType::Action;
                end = i;
                break;
            }
        }

        self.next_slice = end;
        &self.all_args[start..end]
    }
}
