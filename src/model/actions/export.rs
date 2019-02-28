
use crate::model::actions::ActionBuilder;

use clap::{App, Arg, SubCommand};
use crate::model::io;
use crate::model::LQModel;
use crate::model::actions::CLIAction;

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIExport{})
}

struct CLIExport;
impl CLIAction for CLIExport {
    fn name(&self) -> &'static str { "export" }

    fn register_command(&self, app: App<'static, 'static>) -> App<'static, 'static> {

        app.subcommand(SubCommand::with_name("export")
            .about("Save the current model")
            .aliases(&["save", "convert"])
            .arg(Arg::with_name("output")
                .help("Set the output file")
                .required(true)
            )
            .arg(Arg::with_name("format")
                .help("Enforce the export format")
                .short("F")
                .long("format")
            )
        )
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(ExportBuilder::new(model))
    }
}

pub struct ExportBuilder {
    model: LQModel,
    output: Option<String>,
    format: Option<String>,
}


impl ExportBuilder {
    pub fn new(model: LQModel) -> ExportBuilder {
        ExportBuilder{model: model, output: None, format: None}
    }
}

impl ActionBuilder for ExportBuilder {

    fn call(&self) {
        if self.output.is_none() {
            eprintln!("No output file specified");
            return;
        }

        io::save_model(&self.model, &self.output.as_ref().unwrap(), self.format.as_ref().map(|s|&**s));
    }
}
