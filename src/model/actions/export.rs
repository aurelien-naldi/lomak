use crate::model::actions::ActionBuilder;

use crate::model::actions::ArgumentDescr;
use crate::model::actions::CLIAction;
use crate::model::io;
use crate::model::LQModelRef;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec! {
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
    Box::new(CLIExport {})
}

struct CLIExport;
impl CLIAction for CLIExport {
    fn name(&self) -> &'static str {
        "export"
    }

    fn about(&self) -> &'static str {
        "Save the current model"
    }

    fn arguments(&self) -> &'static [ArgumentDescr] {
        &PARAMETERS
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["save", "convert"]
    }

    fn builder(&self, model: LQModelRef) -> Box<dyn ActionBuilder> {
        Box::new(ExportBuilder::new(model))
    }
}

pub struct ExportBuilder {
    model: LQModelRef,
    output: Option<String>,
    format: Option<String>,
}

impl ExportBuilder {
    pub fn new(model: LQModelRef) -> ExportBuilder {
        ExportBuilder {
            model: model,
            output: None,
            format: None,
        }
    }
}

impl ActionBuilder for ExportBuilder {
    fn set_value(&mut self, key: &str, value: &str) {
        match key {
            "output" => self.output = Some(value.to_string()),
            "format" => self.format = Some(value.to_string()),
            _ => (),
        }
    }

    fn call(&self) {
        if self.output.is_none() {
            eprintln!("No output file specified");
            return;
        }

        io::save_model(
            &self.model,
            &self.output.as_ref().unwrap(),
            self.format.as_ref().map(|s| &**s),
        )
        .unwrap();
    }
}
