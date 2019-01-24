pub mod convert;
pub mod repr;
pub mod variables;

use self::repr::expr::Expr;

use std::fmt;

pub trait Grouped {
    fn gfmt(&self, group: &variables::Group, f: &mut fmt::Formatter) -> fmt::Result;
}

pub enum Repr {
    EXPR(Expr),
}

pub struct Formula {
    repr: Repr,
    // TODO: add cache for other formats
}

impl Formula {
    pub fn as_expr(&self) -> Expr {
        match &self.repr {
            Repr::EXPR(e) => e.clone(),
        }
    }

    pub fn set_expr(&mut self, expr: Expr) {
        self.repr = Repr::EXPR(expr);
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Grouped for Formula {
    fn gfmt(&self, group: &variables::Group, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.repr {
            Repr::EXPR(e) => e.gfmt(group, f),
        }
    }
}

pub fn from_expr(expr: Expr) -> Formula {
    Formula {
        repr: Repr::EXPR(expr),
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Repr::EXPR(e) => write!(f, "{}", e),
        }
    }
}
