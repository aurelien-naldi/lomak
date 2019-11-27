use crate::func::expr::Expr;
use crate::model::actions::ActionBuilder;
use crate::model::actions::ArgumentDescr;
use crate::model::actions::CLIAction;
use crate::model::QModel;

use crate::solver;

use crate::func::paths::LiteralSet;
use crate::solver::SolverMode;
use itertools::Itertools;

lazy_static! {
    pub static ref PARAMETERS: Vec<ArgumentDescr> = vec! {
        ArgumentDescr::new("displayed")
            .help("Select output components (comma-separated)")
            .long("displayed")
            .short("d")
            .has_value(true),
    };
}

impl dyn QModel {
    pub fn fixpoints(&'_ self) -> FixedBuilder<'_> {
        FixedBuilder::new(self)
    }
}

pub fn cli_action() -> Box<dyn CLIAction> {
    Box::new(CLIFixed {})
}

struct CLIFixed;
impl CLIAction for CLIFixed {
    fn name(&self) -> &'static str {
        "fixpoints"
    }
    fn about(&self) -> &'static str {
        "Compute the fixed points of the model"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["fixed", "stable", "fp"]
    }

    fn arguments(&self) -> &'static [ArgumentDescr] {
        &PARAMETERS
    }

    fn builder<'a>(&self, model: &'a dyn QModel) -> Box<dyn ActionBuilder + 'a> {
        Box::new(FixedBuilder::new(model))
    }
}

pub struct FixedBuilder<'a> {
    model: &'a dyn QModel,
    restriction: Option<LiteralSet>,
    displayed: Option<Vec<usize>>,
}

impl<'a> FixedBuilder<'a> {
    pub fn new(model: &'a dyn QModel) -> FixedBuilder<'a> {
        FixedBuilder {
            model,
            restriction: None,
            displayed: None,
        }
    }

    pub fn set_displayed_names(&mut self, names: Vec<&str>) {
        if self.displayed.is_none() {
            self.displayed = Some(vec![]);
        }
        let displayed = self.displayed.as_mut().unwrap();
        for name in names {
            // FIXME: multi-valued case
            let uid = self.model.variable_by_name(name);
            match &uid {
                Some(uid) => displayed.push(*uid),
                None => println!("Selected display component not found: {}", name),
            }
        }
    }
}

impl ActionBuilder for FixedBuilder<'_> {
    fn set_value(&mut self, key: &str, value: &str) {
        if let "displayed" = key {
            self.set_displayed_names(value.split(',').collect());
        }
    }

    fn call(&self) {
        let mut solver = solver::get_solver(SolverMode::ALL);
        let s = self
            .model
            .variables()
            .map(|(uid, _)| format!("v{}", uid))
            .join("; ");
        let s = format!("{{{}}}.", s);
        solver.add(&s);

        for (uid, var) in self.model.variables() {
            let cpt = self.model.get_component(var.component);
            let cur = Expr::ATOM(uid);
            let e: Expr = cpt.as_func(var.value);
            for p in cur.not().and(&e).prime_implicants().items() {
                solver.restrict(p);
            }
            for p in cur.and(&e.not()).prime_implicants().items() {
                solver.restrict(p);
            }

            if self.restriction.is_some() {
                solver.restrict(self.restriction.as_ref().unwrap());
            }
        }

        let results = solver.solve();
        match &self.displayed {
            None => {
                let s = self
                    .model
                    .variables()
                    .map(|(uid, _)| self.model.name(uid))
                    .join(" ");
                println!("{}", s);
            }
            Some(dsp) => {
                let s = dsp.iter().map(|uid| self.model.get_name(*uid)).join(" ");
                println!("{}", s);
            }
        }
        for r in results {
            println!("{}", r.filter(&self.displayed));
        }
    }
}
