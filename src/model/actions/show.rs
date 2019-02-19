use crate::model;
use crate::model::actions::Action;
use crate::services::*;


lazy_static! {
    static ref SRV_SHOW: ShowService = ShowService{};
    static ref ALIASES: Vec<String> = vec!(String::from("display"), String::from("print"));
}


pub struct ShowService;

impl Action for ShowService {
    fn run(&self, model: &model::LQModel) {
        print!("{}", model);
    }
}

impl Service for ShowService {
    fn name(&self) -> &str {
        "show"
    }
    fn descr(&self) -> &str {
        "Show the current model"
    }
    fn aliases(&self) -> &Vec<String> {
        &ALIASES
    }
}

