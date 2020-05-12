#[macro_use]
extern crate clap;

use command::SelectedArgs;
use lomak::*;
use model::actions;
use model::io;
use model::modifier;

use clap::{App, AppSettings, ArgMatches};
use lomak::command::CommandContext;
use lomak::model::LQModelRef;
use std::env::ArgsOs;
use std::ffi::OsString;

fn main() {

    let mut app = app_from_crate!()
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::ColoredHelp);

    // Extract slices of arguments for each sub-command
    let mut args_wrapper = SelectedArgs::new();

    // Main CLI parser to handle help messages
    app.get_matches_from(args_wrapper.scan());

    // Apply all commands: loader, modifiers and actions
    let mut context = CommandContext::Empty;
    while args_wrapper.has_next() {
        context = args_wrapper.parse_next(context);
    }
}
