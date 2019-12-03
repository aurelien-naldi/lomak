//! Represent and convert Boolean functions

pub mod convert;
pub mod expr;
pub mod gen;
pub mod paths;
pub mod state;

use self::expr::Expr;
use self::gen::Generator;
use self::paths::Paths;

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use crate::func::state::State;

pub trait VariableNamer {
    /// Write the name corresponding to the given UID
    fn format_name(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result {
        write!(f, "v{}", uid)
    }

    fn as_namer(&self) -> &dyn VariableNamer;

    /// Create a String with the name of the given UID
    fn name(&self, uid: usize) -> String {
        format!("{}", Fmt(|f| self.format_name(f, uid)))
    }
}

pub struct Fmt<F>(pub F)
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Display for Fmt<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

pub trait Grouped {
    fn gfmt(&self, group: &dyn VariableNamer, f: &mut fmt::Formatter) -> fmt::Result;
}

pub struct GroupedTuple<'a, N: VariableNamer, G: Grouped> {
    namer: &'a N,
    val: &'a G,
}

impl<'a, N: VariableNamer, G: Grouped> fmt::Display for GroupedTuple<'a, N, G> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.val.gfmt(self.namer, f)
    }
}

pub struct TrivialNamer {}

impl VariableNamer for TrivialNamer {
    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }
}

/// Supported function representation formats
#[derive(Clone)]
pub enum Repr {
    EXPR(Rc<Expr>),
    GEN(Rc<Generator>),
    PRIMES(Rc<Paths>),
}

/// Common API for all representations of Boolean functions
pub trait BoolRepr {
    /// Wrap this function into a Boolean repr
    fn into_repr(self) -> Repr;

    fn eval(&self, state: &State) -> bool;
}

pub trait FromBoolRepr: BoolRepr {
    fn convert(repr: &Repr) -> Rc<Self>;
    fn is_converted(repr: &Repr) -> bool;
    fn rc_to_repr(rc: Rc<Self>) -> Repr;
}

impl BoolRepr for Repr {
    fn into_repr(self) -> Repr {
        self
    }

    fn eval(&self, state: &State) -> bool {
        match self {
            Repr::EXPR(e) => e.eval(state),
            Repr::PRIMES(p) => p.eval(state),
            Repr::GEN(g) => g.eval(state),
        }
    }
}

/// Carry a function in any supported format
pub struct Formula {
    repr: Repr,
    cached: RefCell<Vec<Repr>>,
}

impl Repr {
    pub fn from<T: BoolRepr>(value: T) -> Repr {
        value.into_repr()
    }

    pub fn convert_as<T: FromBoolRepr>(&self) -> Rc<T> {
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

    pub fn from_bool(value: bool) -> Formula {
        Self::from(Expr::from_bool(value))
    }

    pub fn set<T: BoolRepr>(&mut self, value: T) {
        self.repr = Repr::from(value);
        self.cached.borrow_mut().clear();
    }

    fn cache_repr(&self, repr: Repr) {
        self.cached.borrow_mut().push(repr);
    }

    pub fn convert_as<T: FromBoolRepr>(&self) -> Rc<T> {
        if self.repr.is_a::<T>() {
            return self.repr.convert_as();
        }
        for c in self.cached.borrow().iter() {
            if c.is_a::<T>() {
                return c.convert_as();
            }
        }

        // No matching value found, convert it
        let e: Rc<T> = self.repr.convert_as();
        let r = FromBoolRepr::rc_to_repr(e.clone());
        self.cache_repr(r);
        e
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Grouped for Formula {
    fn gfmt(&self, namer: &dyn VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.repr {
            Repr::EXPR(e) => e.gfmt(namer, f),
            Repr::GEN(g) => g.gfmt(namer, f),
            Repr::PRIMES(p) => write!(f, "{}", p),
        }
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
