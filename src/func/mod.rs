//! Represent, and convert Boolean functions

pub mod convert;
pub mod repr;
pub mod variables;
use self::repr::expr::Expr;
use self::repr::paths::Paths;

use std::fmt;

pub trait Grouped {
    fn gfmt(&self, group: &variables::Group, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Supported function representation formats
pub enum Repr {
    EXPR(Expr),
    PRIMES(Paths),
}

/// Carry a function in any supported format
pub struct Formula {
    repr: Repr,
    cached: Vec<Repr>,
}

impl Repr {

    pub fn is_expr(&self) -> bool {
        match self {
            Repr::EXPR(_) => true,
            _ => false,
        }
    }
    pub fn as_expr(&self) -> Expr {
        match &self {
            Repr::EXPR(e) => e.clone(),
            Repr::PRIMES(p) => p.to_expr(),
        }
    }
}

impl Formula {
    pub fn from_repr(repr: Repr) -> Formula {
        Formula {
            repr: repr,
            cached: vec![],
        }
    }

    pub fn set_repr(&mut self, repr: Repr) {
        self.repr = repr;
        self.cached.clear();
    }

    pub fn from_expr(expr: Expr) -> Formula {
        Self::from_repr(Repr::EXPR(expr))
    }
    pub fn from_primes(p: Paths) -> Formula {
        Self::from_repr(Repr::PRIMES(p))
    }

    pub fn set_expr(&mut self, expr: Expr) {
        self.set_repr(Repr::EXPR(expr));
    }

    pub fn set_primes(&mut self, p: Paths) {
        self.set_repr(Repr::PRIMES(p));
    }

    pub fn as_expr(&self) -> Expr {
        if self.repr.is_expr() {
            return self.repr.as_expr();
        }
        for c in self.cached.iter() {
            if c.is_expr() {
                return c.as_expr();
            }
        }
        return self.repr.as_expr();
    }

    pub fn as_primes(&self) -> Paths {
        match &self.repr {
            Repr::EXPR(e) => e.prime_implicants(),
            Repr::PRIMES(p) => p.clone(),
        }
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
            Repr::PRIMES(p) => write!(f, "{}", p),
        }
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Repr::EXPR(e) => write!(f, "{}", e),
            Repr::PRIMES(p) => write!(f, "{}", p),
        }
    }
}
