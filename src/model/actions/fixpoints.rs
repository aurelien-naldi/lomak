use std::fmt;

use itertools::Itertools;

use crate::func::expr::Expr;
use crate::func::pattern::Pattern;
use crate::model::{GroupedVariables, QModel, SharedModel};
use crate::solver;
use crate::solver::SolverMode;
use std::fmt::Formatter;
use std::ops::Deref;

pub struct FixedBuilder {
    model: SharedModel,
    restriction: Option<Pattern>,
}

pub struct FixedPoints {
    names: Vec<String>,
    patterns: Vec<Pattern>,
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
        let uid = model.get_handle(name);
        if let Some(uid) = uid {
            if self.restriction.is_none() {
                self.restriction = Some(Pattern::new());
            }
            self.restriction.as_mut().unwrap().set(uid, value);
        }
    }

    pub fn solve(&self, max: Option<usize>) -> FixedPoints {
        let mut solver = solver::get_solver(SolverMode::ALL);

        let model = self.model.borrow();

        // Create an ASP variable matching each variable of the model
        let s = model.variables().map(|vid| format!("v{}", vid)).join("; ");
        let s = format!("{{{}}}.", s);
        solver.add(&s);

        // For each variable:
        //   * retrieve the Boolean formula
        //   * derive the stability condition
        //   * encode it in ASP
        for vid in model.variables() {
            let cur = Expr::ATOM(*vid);
            let e = model.get_var_rule(*vid);
            for p in cur.not().and(&e).prime_implicants().iter() {
                solver.restrict(p);
            }
            for p in cur.and(&e.not()).prime_implicants().iter() {
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
    pub fn new(model: &QModel, patterns: Vec<Pattern>) -> Self {
        let names = model
            .variables()
            .map(|vid| model.get_name(*vid).to_string())
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
