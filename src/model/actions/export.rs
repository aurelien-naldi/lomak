
use crate::model;
use crate::services::*;
use crate::model::actions::ActionBuilder;

use clap::SubCommand;
use clap::App;
use crate::model::LQModel;
use crate::model::actions::CLIAction;

/*
        Argument::new("output")
            .long("output")
            .descr("Set the output file")
            .value(true),
        Argument::new("format")
            .long("format")
            .descr("Set the output format")
            .value(true)
*/

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIExport{})
}

struct CLIExport;
impl CLIAction for CLIExport {
    fn name(&self) -> &'static str { "export" }

    fn register_command(&self, mut app: App<'static, 'static>) -> App<'static, 'static> {
        app.subcommand(SubCommand::with_name("export")
            .about("Save the current model")
            .aliases(&["save", "convert"])
        )
    }

    fn builder(&self, model: LQModel) -> Box<dyn ActionBuilder> {
        Box::new(ExportBuilder::new(model))
    }
}

pub struct ExportBuilder;


impl ExportBuilder {
    pub fn new(model: LQModel) -> ExportBuilder {
        ExportBuilder{}
    }
}

impl ActionBuilder for ExportBuilder {

    fn call(&self) {
        // TODO: export the model!!
    }
}
