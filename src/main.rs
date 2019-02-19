#[macro_use]
extern crate clap;

use lomak::*;
use model::actions;
use model::actions::Action;
use model::io;
use services::Service;

use clap::AppSettings;
use clap::SubCommand;

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

    // TODO: add modifiers
    app = app.arg(
        clap::Arg::with_name("prt")
            .short("p")
            .long("perturb")
            .multiple(true)
            .help("[TODO] Apply a perturbation to one or several components")
            .takes_value(true),
    );

    app = app.arg(
        clap::Arg::with_name("mv")
            .long("rename")
            .help("[TODO] Rename a component")
            .multiple(true)
            .takes_value(true),
    );

    for srv in actions::ACTIONS.services.values() {
        let mut cmd = SubCommand::with_name(srv.name())
            .about(srv.descr());
        // Add command aliases
        for alias in srv.aliases() {
            cmd = cmd.alias(alias.as_str());
        }


        // Add command arguments
        for arg in srv.arguments() {
            cmd = cmd.arg(
                clap::Arg::with_name(&arg.name)
                    .takes_value(arg.value)
                    .multiple(arg.multiple)
                    .short(&arg.short)
                    .long(&arg.long),
            );
        }

        app = app.subcommand(cmd);
    }

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

    // TODO: apply selected modifiers in the right order
    let modifiers = vec!["prt", "mv"];
    let mut called_modifiers = vec![];
    for name in modifiers {
        if let Some(ids) = matches.indices_of(name) {
            called_modifiers.push((name, ids));
        }
    }

    for (name, ids) in called_modifiers {
        print!("{}:", name);
        for i in ids {
            print!(" {}", i);
        }
        println!();
    }

    let (s_cmd, _cmd) = matches.subcommand();

    let cur_cmd = actions::ACTIONS.services.get(s_cmd);
    match cur_cmd {
        None => {
            println!("No valid command");
            return;
        }
        Some(a) => {
            a.run(&model);
        }
    }

}
