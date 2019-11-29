//! Generate canonical functions based on the list of signed regulators

use std::collections::HashMap;
use std::rc::Rc;
use std::fmt;

use crate::func;
use crate::func::expr::Expr;
use crate::func::{BoolRepr, Repr};

#[derive(Clone)]
pub enum Sign {
    POSITIVE,
    NEGATIVE,
}

#[derive(Clone)]
pub struct Generator {
    map: HashMap<usize, Sign>,
}

impl Generator {
    /// Generate the corresponding function
    pub fn to_expr(&self) -> Expr {
        let mut expr = Expr::FALSE;
        let mut nexpr = Expr::TRUE;
        for &k in self.map.keys() {
            match self.map.get(&k) {
                None => (),
                Some(Sign::POSITIVE) => expr = expr.and(&Expr::ATOM(k)),
                Some(Sign::NEGATIVE) => nexpr = nexpr.and(&Expr::NATOM(k)),
            }
        }
        expr.and(&nexpr)
    }
}

impl BoolRepr for Generator {
    fn into_repr(self) -> Repr {
        Repr::GEN(Rc::new(self))
    }
}

impl fmt::Display for Generator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for k in self.map.keys() {
            write!(f, "{}{} ", k, self.map.get(k).unwrap())?;
        }
        write!(f, "")
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Sign::POSITIVE => write!(f, ""),
            Sign::NEGATIVE => write!(f, ""),
        }
    }
}

impl func::Grouped for Generator {
    fn gfmt(&self, namer: &dyn func::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        for k in self.map.keys() {
            namer.format_name(f, *k)?;
            write!(f, "{} ", self.map.get(k).unwrap())?;
        }
        write!(f, "")
    }
}
