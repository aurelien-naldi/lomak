//! Represent and convert Boolean functions

pub mod convert;
pub mod variables;
pub mod expr;
pub mod gen;
pub mod paths;

use self::expr::Expr;
use self::gen::Generator;
use self::paths::Paths;

use std::fmt;
use std::cell::RefCell;

pub trait Grouped {
    fn gfmt(&self, group: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Supported function representation formats
#[derive(Clone)]
pub enum Repr {
    EXPR(Expr),
    GEN(Generator),
    PRIMES(Paths),
}

/// Common API for all representations of Boolean functions
pub trait BoolRepr {
    /// Wrap this function into a Boolean repr
    fn into_repr(self) -> Repr;
}

pub trait FromBoolRepr: BoolRepr {
    fn convert(repr: &Repr) -> Self;
    fn is_converted(repr: &Repr) -> bool;
}

impl BoolRepr for Repr {
    fn into_repr(self) -> Repr {
        self
    }
}

/// Carry a function in any supported format
pub struct Formula {
    repr: Repr,
    cached: RefCell<Vec<Repr>>,
}

/// A formula associated with a target value
pub struct Assign {
    pub target: u8,
    pub formula: Formula
}

/// List of formulae forming a full assignment
pub struct Rule {
    assignments: Vec<Assign>
}

impl Repr {

    pub fn from<T: BoolRepr>(value: T) -> Repr {
        value.into_repr()
    }

    /// Test if this function is represented as prime implicants
    pub fn is_primes(&self) -> bool {
        match self {
            Repr::PRIMES(_) => true,
            _ => false,
        }
    }

    pub fn convert_as<T: FromBoolRepr>(&self) -> T {
        T::convert(self)
    }

    pub fn is_a<T: FromBoolRepr>(&self) -> bool {
        T::is_converted(self)
    }
}

impl Formula {
    pub fn from<T: BoolRepr>(value: T) -> Formula {
        Formula {
            repr: Repr::from(value),
            cached: RefCell::new(vec![]),
        }
    }

    pub fn set<T: BoolRepr>(&mut self, value: T) {
        self.repr = Repr::from(value);
        self.cached.borrow_mut().clear();
    }

    fn cache_repr(&self, repr: Repr) {
        self.cached.borrow_mut().push(repr);
    }

    pub fn convert<T: FromBoolRepr>(&self) -> T {

        if self.repr.is_a::<T>() {
            return self.repr.convert_as();
        }
        for c in self.cached.borrow().iter() {
            if c.is_a::<T>() {
                return c.convert_as();
            }
        }

        // No matching value found, convert it
        let e: T = self.repr.convert_as();
        let r = Repr::from(e);
        self.cache_repr(r.clone());
        r.convert_as()
    }
}


impl Rule {
    pub fn from_formula(f: Formula) -> Rule {
        Rule {
            assignments: vec![Assign { target: 1, formula: f }]
        }
    }

    pub fn extend(&mut self, expr: Expr) {
        self.assignments.insert(self.assignments.len(), Assign { target: 1, formula: Formula::from(expr) })
    }

    pub fn set_expr(&mut self, expr: Expr) {
        self.assignments.clear();
        let f = Formula::from(expr);
        self.assignments.insert(0, Assign { target: 1, formula: f });
    }


    pub fn from_repr<T: BoolRepr>(value: T) -> Rule {
        Self::from_formula(Formula::from(value))
    }

    pub fn as_expr(&self) -> Expr {
        if self.assignments.len() < 1 {
            return Expr::FALSE;
        }

        // FIXME: build the expr for target value 1
        self.assignments.get(0).unwrap().convert()
    }
}

impl Assign {

    pub fn convert<T: FromBoolRepr>(&self) -> T {
        self.formula.convert()
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
            Repr::GEN(g) => g.gfmt(namer, f),
            Repr::PRIMES(p) => write!(f, "{}", p),
        }
    }
}

impl Grouped for Rule {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        for a in &self.assignments {
            a.gfmt(namer, f);
        }
        write!(f, "")
    }
}

impl Grouped for Assign {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- ", self.target);
        self.formula.gfmt(namer, f)
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Repr::EXPR(e) => write!(f, "{}", e),
            Repr::GEN(g) => write!(f, "{}", g),
            Repr::PRIMES(p) => write!(f, "{}", p),
        }
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
            writeln!(f, "{}", a);
        }
        write!(f, "")
    }
}
