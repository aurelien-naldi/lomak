//! Rules composing qualitative models

use crate::func::*;
use crate::func::expr::*;

use std::fmt;


/// A formula associated with a target value
pub struct Assign {
    pub target: u8,
    pub formula: Formula
}

/// List of formulae forming a full assignment
pub struct Rule {
    assignments: Vec<Assign>
}


impl Rule {
    pub fn from_formula(f: Formula) -> Rule {
        Rule {
            assignments: vec![Assign { target: 1, formula: f }]
        }
    }

    pub fn from_function<T: BoolRepr>(value: T) -> Rule {
        Self::from_formula(Formula::from(value))
    }

    pub fn extend<T: BoolRepr>(&mut self, value: T) {
        self.assignments.insert(self.assignments.len(), Assign { target: 1, formula: Formula::from(value) })
    }

    pub fn set<T: BoolRepr>(&mut self, value: T) {
        self.assignments.clear();
        let f = Formula::from(value);
        self.assignments.insert(0, Assign { target: 1, formula: f });
    }

    pub fn as_func<T: FromBoolRepr>(&self) -> T {
        if self.assignments.len() < 1 {
            return Expr::FALSE.into_repr().convert_as();
        }

        // FIXME: build the expr for target value 1
        self.assignments.get(0).unwrap().convert()
    }
}

impl Assign {

    pub fn convert<T: FromBoolRepr>(&self) -> T {
        self.formula.convert_as()
    }
}


impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- {}", self.target, self.formula)
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for a in &self.assignments {
            writeln!(f, "{}", a)?;
        }
        write!(f, "")
    }
}

impl Grouped for Rule {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        for a in &self.assignments {
            a.gfmt(namer, f)?;
        }
        write!(f, "")
    }
}

impl Grouped for Assign {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- ", self.target)?;
        self.formula.gfmt(namer, f)
    }
}

