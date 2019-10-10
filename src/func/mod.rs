//! Represent and convert Boolean functions

pub mod convert;
pub mod expr;
pub mod gen;
pub mod paths;

use self::expr::Expr;
use self::gen::Generator;
use self::paths::Paths;

use std::cell::RefCell;
use std::fmt;

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

impl Repr {
    pub fn from<T: BoolRepr>(value: T) -> Repr {
        value.into_repr()
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

    pub fn convert_as<T: FromBoolRepr>(&self) -> T {
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
