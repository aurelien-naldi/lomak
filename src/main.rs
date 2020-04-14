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
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            clap::Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("fmt")
                .short("F")
                .long("format")
                .help("[TODO] Specify the input format")
                .takes_value(true),
        );

    // Extract slices of arguments for each sub-command
    let mut args_wrapper = SelectedArgs::new();

    // Load the selected model
    let matches = app.get_matches_from(args_wrapper.scan());
    let filename = matches.value_of("INPUT").unwrap();
    let format = matches.value_of("fmt");

    let model = match io::load_model(filename, format) {
        Err(e) => {
            println!("ERROR loading \"{}\": {}", filename, e);
            return;
        }
        Ok(m) => m,
    };

    // Apply all modifiers and actions
    let mut context = CommandContext::Model(model);
    while args_wrapper.has_next() {
        context = args_wrapper.parse_next(context);
    }
}
