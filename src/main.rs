#[macro_use]
extern crate clap;

use lomak::*;
use model::actions;
use model::io;
use model::modifier;

use clap::AppSettings;

use log::debug;

fn main() {
    let mut app = app_from_crate!()
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::ColoredHelp)
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
        )
        .arg(
            clap::Arg::with_name("DONE")
                .short("d")
                .long("done")
                .help("Silent: close multiple arguments before the command"),
        );

    // register available modifiers and commands
    app = modifier::register_modifiers(app);
    app = actions::register_commands(app);

    let matches = app.get_matches();

    let filename = matches.value_of("INPUT").unwrap();
    let format = matches.value_of("fmt");

    debug!("Loading input file: {}", filename);
    let model = match io::load_model(filename, format) {
        Err(e) => {
            println!("ERROR loading \"{}\": {}", filename, e);
            return;
        }
        Ok(m) => m,
    };

    // Apply the selected modifiers
    let model = modifier::modify(model, &matches);

    // Call the selected command with its parameters
    let (s_cmd, subcmd) = matches.subcommand();
    if subcmd.is_some() {
        actions::run_command(s_cmd, subcmd.unwrap(), model);
    }

}
