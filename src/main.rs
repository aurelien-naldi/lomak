#[macro_use]
extern crate clap;

use clap::AppSettings;

use command::SelectedArgs;
use lomak::command::CommandContext;
use lomak::*;

fn main() {
    let app = app_from_crate!()
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::ColoredHelp);

    // Extract slices of arguments for each sub-command
    let mut args_wrapper = SelectedArgs::new();

    // Main CLI parser to handle help messages
    app.get_matches_from(args_wrapper.scan());

    // Apply all commands: loader, modifiers and actions
    let mut context = CommandContext::default();
    while args_wrapper.has_next() {
        match args_wrapper.parse_next(&mut context) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
