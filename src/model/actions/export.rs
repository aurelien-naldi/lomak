use crate::model;
use crate::model::actions::Action;
use crate::services::*;
use crate::model::io;

lazy_static! {
    static ref SRV_EXPORT: ExportService = ExportService{};
    static ref ARGUMENTS: Vec<Argument> = vec!(
        Argument::new("output")
            .long("output")
            .descr("Set the output file")
            .value(true),
        Argument::new("format")
            .long("format")
            .descr("Set the output format")
            .value(true)
    );
}

pub struct ExportService;

impl Action for ExportService {
    fn run(&self, model: &model::LQModel) {
        // FIXME: retrieve parameters
        io::save_model(model, "", None);
    }
}

impl Service for ExportService {
    fn name(&self) -> &str {
        "export"
    }

    fn descr(&self) -> &str {
        "Save the current model"
    }

    fn arguments(&self) -> &Vec<Argument> {
        &ARGUMENTS
    }
}

