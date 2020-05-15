use std::fmt;
use std::rc::Rc;

use itertools::Itertools;

use crate::func::expr::Expr;
use crate::func::paths::LiteralSet;
use crate::model::{QModel, SharedModel};
use crate::solver;
use crate::solver::SolverMode;
use std::fmt::Formatter;
use std::ops::Deref;

pub struct FixedBuilder {
    model: SharedModel,
    restriction: Option<LiteralSet>,
}

pub struct FixedPoints {
    names: Vec<String>,
    patterns: Vec<LiteralSet>,
    displayed: Option<Vec<usize>>,
}

impl FixedBuilder {
    pub fn new(model: SharedModel) -> Self {
        FixedBuilder {
            model,
            restriction: None,
        }
    }

    /// Apply additional restrictions to the search for fixed points
    pub fn restrict_by_name(&mut self, name: &str, value: bool) {
        let model = self.model.borrow();
        let uid = model.variable_by_name(name);
        if let Some(uid) = uid {
            if self.restriction.is_none() {
                self.restriction = Some(LiteralSet::new());
            }
            self.restriction.as_mut().unwrap().set_literal(uid, value);
        }
    }

    pub fn solve(&self, max: Option<usize>) -> FixedPoints {
        let mut solver = solver::get_solver(SolverMode::ALL);

        let model = self.model.borrow();

        // Create an ASP variable matching each variable of the model
        let s = model
            .variables()
            .map(|(uid, _)| format!("v{}", uid))
            .join("; ");
        let s = format!("{{{}}}.", s);
        solver.add(&s);

        // For each variable:
        //   * retrieve the Boolean formula
        //   * derive the stability condition
        //   * encode it in ASP
        for (uid, var) in model.variables() {
            let cpt = model.get_component_ref(var.component);
            let cpt = cpt.borrow();
            let cur = Expr::ATOM(uid);
            let e: Rc<Expr> = cpt.get_formula(var.value).convert_as();
            for p in cur.not().and(&e).prime_implicants().items() {
                solver.restrict(p);
            }
            for p in cur.and(&e.not()).prime_implicants().items() {
                solver.restrict(p);
            }
        }

        // Add additional restrictions
        if self.restriction.is_some() {
            solver.restrict(self.restriction.as_ref().unwrap());
        }

        // Extract patterns from the clingo results
        let patterns = solver
            .solve()
            .into_iter()
            .map(|r| r.as_pattern())
            .take(max.unwrap_or(10000))
            .collect_vec();

        FixedPoints::new(model.deref(), patterns)
    }
}

impl FixedPoints {
    pub fn new(model: &dyn QModel, patterns: Vec<LiteralSet>) -> Self {
        let names = model
            .variables()
            .into_iter()
            .map(|(u, _)| model.get_name(u))
            .collect_vec();
        FixedPoints {
            names: names,
            patterns: patterns,
            displayed: None,
        }
    }

    pub fn set_displayed_names(&mut self, names: Option<Vec<String>>) {
        match names {
            None => self.set_displayed(None),
            Some(names) => {
                let selected = names
                    .iter()
                    .filter_map(|n| self.names.iter().position(|name| n == name))
                    .collect_vec();
                if selected.len() > 0 {
                    self.set_displayed(Some(selected));
                }
            }
        }
    }

    pub fn set_displayed(&mut self, displayed: Option<Vec<usize>>) {
        self.displayed = displayed;
    }
}

impl fmt::Display for FixedPoints {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = match &self.displayed {
            None => self.names.iter().enumerate().map(|(u, _)| u).collect_vec(),
            Some(v) => v.clone(),
        };
        writeln!(
            f,
            "{}",
            display
                .iter()
                .map(|uid| self.names.get(*uid).unwrap())
                .join(" ")
        )?;

        for p in self.patterns.iter() {
            p.filter_fmt(f, &display)?;
            writeln!(f, "")?;
        }
        write!(f, "")
    }
}
