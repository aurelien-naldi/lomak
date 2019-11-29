use crate::model::actions::ActionBuilder;
use crate::model::actions::ArgumentDescr;

use crate::model::actions::CLIAction;
use crate::model::QModel;
use crate::func::expr::Expr;

use std::rc::Rc;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec! {
        ArgumentDescr::new("booleanized")
            .help("Show Booleanized functions")
            .long("bool")
            .short("b")
    };
}

struct CLIShow;

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIShow {})
}

impl CLIAction for CLIShow {
    fn name(&self) -> &'static str {
        "show"
    }
    fn about(&self) -> &'static str {
        "Show the current model"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["display", "print"]
    }

    fn arguments(&self) -> &'static [ArgumentDescr] {
        &PARAMETERS
    }

    fn builder<'a>(&self, model: &'a dyn QModel) -> Box<dyn ActionBuilder + 'a> {
        Box::new(ShowBuilder::new(model))
    }
}

pub struct ShowBuilder<'a> {
    model: &'a dyn QModel,
    booleanized: bool,
}

impl<'a> ShowBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> ShowBuilder<'a> {
        ShowBuilder { model, booleanized: false }
    }
}

impl ActionBuilder for ShowBuilder<'_> {
    fn set_flag(&mut self, flag: &str) {
        match flag {
            "booleanized" => self.booleanized = true,
            _ => eprintln!("This action has no flag '{}'", flag),
        }
    }

    fn call(&self) {
        if self.booleanized {
            for (uid, var) in self.model.variables() {
                let cpt = self.model.get_component(var.component);
                let e: Rc<Expr> = cpt.as_func(var.value);

                println!("{}: {},{} => {}", uid, cpt.name, var.value, e);
            }
        } else {
            println!("{}", self.model.for_display());
        }
        println!();
    }
}
