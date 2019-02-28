//! Represent, and convert Boolean functions

pub mod convert;
pub mod variables;
pub mod expr;
pub mod paths;

use self::expr::Expr;
use self::paths::Paths;

use std::fmt;
use std::cell::RefCell;

pub trait Grouped {
    fn gfmt(&self, group: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Supported function representation formats
pub enum Repr {
    EXPR(Expr),
    PRIMES(Paths),
}

/// Carry a function in any supported format
pub struct Formula {
    repr: Repr,
    cached: RefCell<Vec<Repr>>,
}

impl Repr {

    /// Test if this function is represented as an expression
    pub fn is_expr(&self) -> bool {
        match self {
            Repr::EXPR(_) => true,
            _ => false,
        }
    }

    /// Convert this function into an expression
    pub fn as_expr(&self) -> Expr {
        match &self {
            Repr::EXPR(e) => e.clone(),
            Repr::PRIMES(p) => p.to_expr(),
        }
    }

    /// Test if this function is represented as prime implicants
    pub fn is_primes(&self) -> bool {
        match self {
            Repr::PRIMES(_) => true,
            _ => false,
        }
    }

    /// Convert this function into a list of prime implicants
    pub fn as_primes(&self) -> Paths {
        match &self {
            Repr::PRIMES(p) => p.clone(),
            Repr::EXPR(e) => e.prime_implicants(),
        }
    }
}

impl Formula {
    pub fn from_repr(repr: Repr) -> Formula {
        Formula {
            repr: repr,
            cached: RefCell::new(vec![]),
        }
    }

    pub fn set_repr(&mut self, repr: Repr) {
        self.repr = repr;
        self.cached.borrow_mut().clear();
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

    fn cache_repr(&self, repr: Repr) {
        self.cached.borrow_mut().push(repr);
    }

    pub fn as_expr(&self) -> Expr {
        if let Repr::EXPR(e) = &self.repr {
            return e.clone();
        }
        for c in self.cached.borrow().iter() {
            if let Repr::EXPR(e) = c {
                return e.clone();
            }
        }

        // No matching value found, convert it
        let e = self.repr.as_expr();
        self.cache_repr(Repr::EXPR(e.clone()));
        e
    }

    pub fn as_primes(&self) -> Paths {
        if let Repr::PRIMES(p) = &self.repr {
            return p.clone();
        }
        for c in self.cached.borrow().iter() {
            if let Repr::PRIMES(p) = c {
                return p.clone();
            }
        }

        // No matching value found, convert it
        let p = self.repr.as_primes();
        self.cache_repr(Repr::PRIMES(p.clone()));
        p
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Grouped for Formula {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.repr {
            Repr::EXPR(e) => e.gfmt(namer, f),
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
