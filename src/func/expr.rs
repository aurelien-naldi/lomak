use core::ops::BitAnd;
use core::ops::BitOr;
use core::ops::Not;
use std::fmt;
use std::iter::Iterator;
use std::rc::Rc;

use crate::func;
use crate::func::pattern::Pattern;
use crate::func::state::State;
use crate::func::*;
use crate::helper::error::{GenericError, ParseError};
use crate::variables::GroupedVariables;

/* ************************************************************************************* */
/* ************************ Data structures and basic operations *********************** */
/* ************************************************************************************* */

#[derive(Clone, PartialEq)]
pub enum Expr {
    TRUE,
    FALSE,
    ATOM(usize),
    NATOM(usize),
    OPER(Operator, Children),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Operator {
    AND,
    OR,
    NAND,
    NOR,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Comparator {
    EQ,
    NEQ,
    GT,
    GEQ,
    LT,
    LEQ,
}

impl Comparator {
    pub fn get_expr(
        &self,
        grp: &impl GroupedVariables,
        var: usize,
        val: usize,
    ) -> Result<Expr, ParseError> {
        let (min, max) = match self {
            Comparator::EQ => {
                if val == 0 {
                    (None, Some(1))
                } else {
                    (Some(val), Some(val + 1))
                }
            }
            Comparator::NEQ => {
                if val > 0 {
                    if let Some(n) = grp.get_variable(var, val + 1) {
                        // The next value exists and the current one is at least 1, so it must exist
                        let c = grp.get_variable(var, val).unwrap();
                        return Ok(Expr::ATOM(n).or(&Expr::NATOM(c)));
                    }
                }
                (Some(val + 1), None)
            }
            Comparator::GEQ => (Some(val), None),
            Comparator::LEQ => (None, Some(val + 1)),
            Comparator::GT => (Some(val + 1), None),
            Comparator::LT => (None, Some(val)),
        };

        let emin = min.map(|v| grp.get_variable(var, v).map(Expr::ATOM));
        let emax = max.map(|v| grp.get_variable(var, v).map(Expr::NATOM));

        match (emin, emax) {
            (Some(Some(mn)), Some(Some(mx))) => Ok(mn.and(&mx)),
            (Some(Some(e)), _) => Ok(e),
            (_, Some(Some(e))) => Ok(e),
            _ => Err(GenericError::new("Could not construct a constraint!".to_owned()).into()),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Children {
    pub data: Rc<Vec<Expr>>,
}

/// Helper trait to provide replacement expressions for atoms in logical expressions.
///
/// This trait is used through [`Expr::replace_variables`] to eliminate a variable from an expression.
pub trait AtomReplacer {
    /// Provide a replacement expression if available.
    /// Returns None if no change are needed.
    fn replace(&mut self, var: usize, value: bool) -> Option<Expr>;
}

impl Expr {
    pub fn from_bool(b: bool) -> Self {
        if b {
            Expr::TRUE
        } else {
            Expr::FALSE
        }
    }

    pub fn not(&self) -> Self {
        match self {
            Expr::TRUE => Expr::FALSE,
            Expr::FALSE => Expr::TRUE,
            Expr::ATOM(u) => Expr::NATOM(*u),
            Expr::NATOM(u) => Expr::ATOM(*u),
            Expr::OPER(o, c) => Expr::OPER(o.not(), c.clone()),
        }
    }

    pub fn and(&self, e: &Expr) -> Self {
        Operator::AND.binary(self, e)
    }

    pub fn or(&self, e: &Expr) -> Self {
        Operator::OR.binary(self, e)
    }
}

impl BoolRepr for Expr {
    fn into_repr(self) -> Repr {
        Repr::EXPR(Rc::new(self))
    }

    fn eval(&self, state: &State) -> bool {
        match self {
            Expr::TRUE => true,
            Expr::FALSE => false,
            Expr::ATOM(u) => state.contains(*u),
            Expr::NATOM(u) => !state.contains(*u),
            Expr::OPER(op, children) => op.eval(children, state),
        }
    }
}

impl FromBoolRepr for Expr {
    fn convert(repr: &Repr) -> Rc<Self> {
        match repr {
            Repr::EXPR(e) => e.clone(),
            Repr::GEN(g) => Rc::new(g.to_expr()),
            Repr::PRIMES(p) => Rc::new(p.to_expr()),
        }
    }

    fn is_converted(repr: &Repr) -> bool {
        matches!(repr, Repr::EXPR(_))
    }

    fn rc_to_repr(rc: Rc<Self>) -> Repr {
        Repr::EXPR(rc)
    }
}

impl Operator {
    fn empty(self) -> Expr {
        match self {
            Operator::AND => Expr::TRUE,
            Operator::NAND => Expr::FALSE,
            Operator::OR => Expr::FALSE,
            Operator::NOR => Expr::TRUE,
        }
    }

    pub fn binary(self, a: &Expr, b: &Expr) -> Expr {
        self.join(&mut vec![a.clone(), b.clone()].into_iter())
    }

    // TODO: ensure that all atoms in a complex expression use the same variable group?
    pub fn join(self, iter: &mut dyn Iterator<Item = Expr>) -> Expr {
        let children = Children::from_expressions(iter);
        // TODO: filter unnecessary elements
        match children.len() {
            0 => self.empty(),
            1 => Expr::clone(&children.data[0]),
            _ => Expr::OPER(self, children),
        }
    }

    fn not(self) -> Self {
        match self {
            Operator::AND => Operator::NAND,
            Operator::NAND => Operator::AND,
            Operator::OR => Operator::NOR,
            Operator::NOR => Operator::OR,
        }
    }

    fn negate(self, neg: bool) -> Self {
        if neg {
            self.not()
        } else {
            self
        }
    }

    fn propagate_not(self, neg: bool) -> (Self, bool) {
        match self.negate(neg) {
            Operator::AND => (Operator::AND, false),
            Operator::OR => (Operator::OR, false),
            Operator::NAND => (Operator::OR, true),
            Operator::NOR => (Operator::AND, true),
        }
    }

    fn priority(self) -> u8 {
        match self {
            Operator::AND => 2,
            Operator::NAND => 2,
            Operator::OR => 1,
            Operator::NOR => 1,
        }
    }

    fn is_neg(self) -> bool {
        match self {
            Operator::AND => false,
            Operator::OR => false,
            Operator::NAND => true,
            Operator::NOR => true,
        }
    }

    fn is_disjunction(self) -> bool {
        match self {
            Operator::AND => false,
            Operator::OR => true,
            Operator::NAND => false,
            Operator::NOR => true,
        }
    }

    fn eval(self, children: &Children, state: &State) -> bool {
        match self {
            Operator::AND => {
                for c in children.data.iter() {
                    if !c.eval(state) {
                        return false;
                    }
                }
                true
            }
            Operator::OR => {
                for c in children.data.iter() {
                    if c.eval(state) {
                        return true;
                    }
                }
                false
            }
            Operator::NAND => !Operator::AND.eval(children, state),
            Operator::NOR => !Operator::OR.eval(children, state),
        }
    }
}

/*  ******************** Manipulate the list of children ******************** */
impl Children {
    pub fn from_expressions(iter: &mut dyn Iterator<Item = Expr>) -> Children {
        Children {
            data: Rc::new(iter.collect()),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/* ************************************************************************************* */
/*                              Replace parts of the function                            */
/* ************************************************************************************* */
impl Expr {
    /// Replace some variables with new sub-functions when needed.
    /// The replacer parameter provides the replacement subfunctions for individual variables.
    ///
    /// Returns Some(result) if at least one variable was changed, None otherwise
    pub fn replace_variables(&self, replacer: &mut impl AtomReplacer) -> Option<Self> {
        match self {
            Expr::TRUE => None,
            Expr::FALSE => None,
            Expr::ATOM(u) => replacer.replace(*u, true),
            Expr::NATOM(u) => replacer.replace(*u, false),
            Expr::OPER(o, c) => {
                let mut has_changed = false;
                let mut new_children: Vec<Expr> = Vec::new();
                for child in c.data.iter() {
                    match child.replace_variables(replacer) {
                        None => new_children.push(child.clone()),
                        Some(c) => {
                            new_children.push(c);
                            has_changed = true;
                        }
                    }
                }

                if has_changed {
                    return Some(Expr::OPER(
                        *o,
                        Children {
                            data: Rc::new(new_children),
                        },
                    ));
                }
                None
            }
        }
    }
}

/* ************************************************************************************* */
/*                                  Simplification and NNF                               */
/* ************************************************************************************* */
impl Expr {
    pub fn simplify(&self) -> Option<Expr> {
        match self {
            Expr::TRUE => None,
            Expr::FALSE => None,
            _ => self._simplify(false, false),
        }
    }

    pub fn nnf(&self) -> Option<Expr> {
        match self {
            Expr::TRUE => None,
            Expr::FALSE => None,
            _ => self._simplify(false, true),
        }
    }

    fn _simplify(&self, neg: bool, nnf: bool) -> Option<Expr> {
        match self {
            // Always return a "changed" result for true/false to enforce the parent update
            Expr::TRUE => Some(if neg { self.not() } else { self.clone() }),
            Expr::FALSE => Some(if neg { self.not() } else { self.clone() }),
            Expr::ATOM(_u) => {
                if neg {
                    Some(self.not())
                } else {
                    None
                }
            }
            Expr::NATOM(_u) => {
                if neg {
                    Some(self.not())
                } else {
                    None
                }
            }
            Expr::OPER(o, c) => c.simplify(*o, neg, nnf),
        }
    }
}

impl Children {
    // TODO: merge nested operations
    fn simplify(&self, op: Operator, neg: bool, nnf: bool) -> Option<Expr> {
        let mut simplified = false;
        let mut children = Vec::with_capacity(self.data.len());

        // propagate negations when performing NNF simplification
        let (next_op, next_neg) = if nnf {
            op.propagate_not(neg)
        } else {
            (op, neg)
        };

        let disjunction = next_op.is_disjunction();
        for child in self.data.iter() {
            match child._simplify(next_neg, nnf) {
                None => children.push(child.clone()),
                Some(e) => {
                    match e {
                        Expr::TRUE => {
                            if disjunction {
                                return Some(Expr::TRUE);
                            }
                        }
                        Expr::FALSE => {
                            if !disjunction {
                                return Some(Expr::FALSE);
                            }
                        }
                        _ => children.push(e.clone()),
                    }
                    simplified = true;
                }
            }
        }

        if !simplified && !neg && !next_neg {
            return None;
        }

        match children.len() {
            0 => Some(next_op.empty()),
            1 => Some(Expr::clone(&children[0])),
            _ => Some(Expr::OPER(
                next_op,
                Children {
                    data: Rc::new(children),
                },
            )),
        }
    }
}

/* ************************************************************************************* */
/* ********************************** Manage literals ********************************** */
/* ************************************************************************************* */

impl Expr {
    #[allow(dead_code)]
    pub fn replace_literal(&self, f: &dyn Fn(usize, bool) -> Option<Expr>) -> Option<Expr> {
        match self {
            Expr::TRUE => None,
            Expr::FALSE => None,
            Expr::ATOM(u) => f(*u, false),
            Expr::NATOM(u) => f(*u, true),
            Expr::OPER(o, c) => c.replace_literal(f, *o),
        }
    }

    #[allow(dead_code)]
    pub fn contains_literal(&self, uid: usize, neg: bool) -> bool {
        match self {
            Expr::TRUE => false,
            Expr::FALSE => false,
            Expr::ATOM(u) => !neg && uid == *u,
            Expr::NATOM(u) => neg && uid == *u,
            Expr::OPER(o, c) => c.contains_literal(uid, o.is_neg() == neg),
        }
    }

    pub fn get_literals(&self) -> Pattern {
        let mut p = Pattern::new();
        self._fill_literals(&mut p, false);
        p
    }

    pub fn _fill_literals(&self, p: &mut Pattern, neg: bool) {
        match self {
            Expr::TRUE => (),
            Expr::FALSE => (),
            Expr::ATOM(u) => p.set_ignoring_conflicts(*u, !neg),
            Expr::NATOM(u) => p.set_ignoring_conflicts(*u, neg),
            Expr::OPER(o, c) => c.fill_literals(p, o.is_neg() != neg),
        }
    }
}

impl Children {
    fn replace_literal(
        &self,
        f: &dyn Fn(usize, bool) -> Option<Expr>,
        op: Operator,
    ) -> Option<Expr> {
        let children: Vec<Option<Expr>> = self.data.iter().map(|c| c.replace_literal(f)).collect();
        let count = children.iter().filter(|c| c.is_some()).count();
        if count > 1 {
            let children = self
                .data
                .iter()
                .zip(children.into_iter())
                .map(|(c, r)| match r {
                    Some(e) => e,
                    None => c.clone(),
                });
            return Some(Expr::OPER(
                op,
                Children {
                    data: Rc::new(children.collect()),
                },
            ));
        }
        None
    }

    pub fn contains_literal(&self, uid: usize, neg: bool) -> bool {
        for child in self.data.iter() {
            if child.contains_literal(uid, neg) {
                return true;
            }
        }
        false
    }

    pub fn fill_literals(&self, p: &mut Pattern, neg: bool) {
        for child in self.data.iter() {
            child._fill_literals(p, neg);
        }
    }
}

/* ************************************************************************************* */
/* ************************************* Formatting ************************************ */
/* ************************************************************************************* */

struct FormatContext<'a> {
    parent_priority: u8,
    group: Option<&'a dyn VariableNamer>,
}

impl<'a> FormatContext<'a> {
    fn new() -> FormatContext<'a> {
        FormatContext {
            parent_priority: 0,
            group: None,
        }
    }

    fn from_group(grp: &dyn VariableNamer) -> FormatContext {
        FormatContext {
            parent_priority: 0,
            group: Some(grp),
        }
    }

    fn update_priority(&mut self, op: Operator) -> bool {
        let old = self.parent_priority;
        self.parent_priority = op.priority();
        self.parent_priority > old
    }

    fn write_var(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result {
        match &self.group {
            None => write!(f, "v{}", uid),
            Some(g) => g.format_name(f, uid),
        }
    }
}

pub struct NamedExpr<'a> {
    pub expr: &'a Expr,
    pub namer: &'a dyn VariableNamer,
}

impl<'a> fmt::Display for NamedExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.expr.gfmt(self.namer, f)
    }
}

impl func::Grouped for Expr {
    fn gfmt(&self, namer: &dyn VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        let mut context = FormatContext::from_group(namer);
        self._fmt(f, &mut context)
    }
}

// Formatting of functions: use an internal function to add the needed parenthesis
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut context = FormatContext::new();
        self._fmt(f, &mut context)
    }
}
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self)
    }
}
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::AND => write!(f, "&"),
            Operator::NAND => write!(f, "&"),
            Operator::OR => write!(f, "|"),
            Operator::NOR => write!(f, "|"),
        }
    }
}

impl Expr {
    // Helper function for the formatter
    fn _fmt(&self, f: &mut fmt::Formatter, context: &mut FormatContext) -> fmt::Result {
        match self {
            Expr::TRUE => write!(f, "True"),
            Expr::FALSE => write!(f, "False"),
            Expr::ATOM(v) => context.write_var(f, *v),
            Expr::NATOM(v) => {
                write!(f, "!")?;
                context.write_var(f, *v)
            }
            Expr::OPER(o, c) => c.format(f, *o, context),
        }
    }
}

impl Children {
    fn format(
        &self,
        f: &mut fmt::Formatter,
        op: Operator,
        context: &mut FormatContext,
    ) -> fmt::Result {
        let mut n = self.data.len();
        if n < 1 {
            return write!(f, "[]");
        }

        let need_paren = context.update_priority(op);

        let mut prefix = "";
        let mut postfix = "";
        if op.is_neg() {
            prefix = "!(";
            postfix = ")";
        } else if need_paren {
            prefix = "(";
            postfix = ")";
        }

        write!(f, "{}", prefix)?;
        n -= 1;
        for idx in 0..n {
            let r = self.data[idx]._fmt(f, context);
            if r.is_err() {
                return r;
            }
            write!(f, " {} ", op)?;
        }
        self.data[n]._fmt(f, context)?;
        write!(f, "{}", postfix)
    }
}

/*
 * Overload operators to write readable expressions
 */

impl Not for Expr {
    type Output = Self;
    fn not(self) -> Self {
        Expr::not(&self)
    }
}
impl<'a> Not for &'a Expr {
    type Output = Expr;
    fn not(self) -> Expr {
        Expr::not(self)
    }
}

impl BitAnd for Expr {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.and(&rhs)
    }
}

impl<'a> BitAnd<&'a Expr> for Expr {
    type Output = Self;
    fn bitand(self, rhs: &Self) -> Self {
        self.and(rhs)
    }
}

impl<'a> BitAnd<Expr> for &'a Expr {
    type Output = Expr;
    fn bitand(self, rhs: Expr) -> Expr {
        self.and(&rhs)
    }
}

impl<'a> BitAnd<&'a Expr> for &'a Expr {
    type Output = Expr;
    fn bitand(self, rhs: Self) -> Expr {
        self.and(rhs)
    }
}

impl BitOr for Expr {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.or(&rhs)
    }
}

impl<'a> BitOr<&'a Expr> for Expr {
    type Output = Self;
    fn bitor(self, rhs: &Self) -> Self {
        self.or(rhs)
    }
}

impl<'a> BitOr<Expr> for &'a Expr {
    type Output = Expr;
    fn bitor(self, rhs: Expr) -> Expr {
        self.or(&rhs)
    }
}
impl<'a> BitOr<&'a Expr> for &'a Expr {
    type Output = Expr;
    fn bitor(self, rhs: Self) -> Expr {
        self.or(rhs)
    }
}

/* ************************************* TODO: TESTS *********************************** */

#[cfg(test)]
mod tests {
    use crate::func::implicant::Implicants;
    use crate::func::*;

    use super::*;

    #[test]
    fn conj_extension() {
        let grp = TrivialNamer {};
        let a = Expr::ATOM(1);
        let b = Expr::ATOM(2);
        let c = Expr::ATOM(3);

        let x = a & b | c;

        let s = x.simplify();
        assert_eq!(s, None);
        assert_eq!(4, 4);
    }

    #[test]
    fn simplification() {
        let a = Expr::ATOM(1);
        let b = Expr::ATOM(2);

        let expr = a.or(&b).not();
        assert_eq!(expr.simplify(), None);

        let c_expr = Expr::FALSE.or(&expr);
        let s_expr = c_expr.simplify();
        assert_ne!(s_expr, None);

        assert_eq!(s_expr.unwrap(), expr);

        let n_expr = a.not().and(&b.not());
        assert_eq!(c_expr.nnf().unwrap(), n_expr);
    }

    #[test]
    fn evaluation() {
        let a = Expr::ATOM(1);
        let b = Expr::ATOM(2);
        let c = Expr::ATOM(3);

        let expr = a.or(&b).not().and(&c.not()).or(&a);
        let path: Rc<Implicants> = expr.clone().into_repr().convert_as();

        let mut state = State::new();
        assert_eq!(expr.eval(&state), true);
        assert_eq!(path.eval(&state), true);
        state.insert(2);
        assert_eq!(expr.eval(&state), false);
        assert_eq!(path.eval(&state), false);
        state.insert(3);
        assert_eq!(expr.eval(&state), false);
        assert_eq!(path.eval(&state), false);
        state.insert(1);
        assert_eq!(expr.eval(&state), true);
        assert_eq!(path.eval(&state), true);
    }
}
