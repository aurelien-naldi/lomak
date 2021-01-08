use std::fmt;

use itertools::Itertools;

use crate::func::expr::Expr;
use crate::func::pattern::Pattern;
use crate::func::Formula;
use crate::helper::error::EmptyLomakResult;
use crate::helper::solver;
use crate::helper::solver::SolverMode;
use crate::model::{GroupedVariables, SharedModel};
use crate::variables::ModelVariables;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::rc::Rc;

pub struct FixedBuilder {
    variables: Rc<ModelVariables>,
    rules: Rc<HashMap<usize, Formula>>,
    restriction: Option<Pattern>,
}

pub struct FixedPoints {
    variables: Rc<ModelVariables>,
    patterns: Vec<Pattern>,
    displayed: Option<Vec<usize>>,
}

impl FixedBuilder {
    pub fn new(model: SharedModel) -> Self {
        let m = model.borrow();
        FixedBuilder {
            variables: m.frozen_variables(),
            rules: m.frozen_rules(),
            restriction: None,
        }
    }

    /// Apply additional restrictions to the search for fixed points
    pub fn restrict_by_name(&mut self, name: &str, value: bool) -> EmptyLomakResult {
        let uid = self.variables.get_handle_res(name)?;
        self._restrict(uid, value);
        Ok(())
    }

    /// Apply additional restrictions to the search for fixed points
    pub fn _restrict(&mut self, uid: usize, value: bool) {
        if self.restriction.is_none() {
            self.restriction = Some(Pattern::new());
        }
        self.restriction.as_mut().unwrap().set(uid, value);
    }

    pub fn solve(&self, max: Option<usize>) -> FixedPoints {
        let mut solver = solver::get_solver(SolverMode::ALL);

        // Create an ASP variable matching each variable of the model
        let s = self
            .variables
            .variables()
            .map(|vid| format!("v{}", vid))
            .join("; ");
        let s = format!("{{{}}}.", s);
        println!("#VARS: {{{}}}.", s);
        solver.add(&s);

        // For each variable:
        //   * retrieve the Boolean formula
        //   * derive the stability condition
        //   * encode it in ASP
        for vid in self.variables.variables() {
            let cur = Expr::ATOM(*vid);
            // TODO: handle missing expr ??
            let e: Rc<Expr> = self.rules.get(vid).map(|f| f.convert_as()).unwrap();
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

        FixedPoints::new(Rc::clone(&self.variables), patterns)
    }
}

impl FixedPoints {
    pub fn new(variables: Rc<ModelVariables>, patterns: Vec<Pattern>) -> Self {
        FixedPoints {
            variables: variables,
            patterns: patterns,
            displayed: None,
        }
    }

    pub fn set_displayed_names(&mut self, names: Option<Vec<String>>) {
        self.displayed = match names {
            None => None,
            Some(n) => Some(
                n.iter()
                    .filter_map(|s| self.variables.get_handle(s))
                    .collect(),
            ),
        };
    }

    pub fn set_displayed(&mut self, displayed: Option<Vec<usize>>) {
        self.displayed = displayed;
    }
}

impl fmt::Display for FixedPoints {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: filter displayed components
        writeln!(
            f,
            "{}",
            self.variables
                .variables()
                .map(|uid| self.variables.get_name(*uid))
                .join(" ")
        )?;

        for p in self.patterns.iter() {
            writeln!(f, "{}", p)?;
        }
        write!(f, "")
    }
}
