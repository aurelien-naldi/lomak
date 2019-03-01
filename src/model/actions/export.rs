
use crate::model::actions::ActionBuilder;

use crate::model::io;
use crate::model::LQModel;
use crate::model::actions::CLIAction;
use crate::model::actions::ArgumentDescr;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec!{
        ArgumentDescr::new("output")
            .help("Set the output file")
            .required(true),
        ArgumentDescr::new("format")
            .help("Enforce the output format")
            .long("format")
            .short("F")
            .has_value(true),
    };
}

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIExport{})
}

struct CLIExport;
impl CLIAction for CLIExport {
    fn name(&self) -> &'static str { "export" }
    fn about(&self) -> &'static str { "Save the current model" }

    fn aliases(&self) -> &'static[&'static str] {
        &["save", "convert"]
    }

    fn arguments(&self) -> &'static[ArgumentDescr] {
        &PARAMETERS
    }

    /*
            cmd = cmd.arg(Arg::with_name("output")
                    .help("Set the output file")
                    .required(true)
                );
            cmd = cmd.arg(Arg::with_name("format")
                    .help("Enforce the export format")
                    .short("F")
                    .long("format")
                );
    */

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

    fn set_args(&mut self, args: &clap::ArgMatches) {
        self.output = args.value_of("output").map(|s|s.to_string());
        self.format = args.value_of("format").map(|s|s.to_string());
    }

    fn call(&self) {
        if self.output.is_none() {
            eprintln!("No output file specified");
            return;
        }

        io::save_model(&self.model, &self.output.as_ref().unwrap(), self.format.as_ref().map(|s|&**s));
    }
}
