use std::fmt;
use std::iter::Iterator;
use std::rc::Rc;

use core::ops::BitAnd;
use core::ops::BitOr;
use core::ops::Not;

use crate::func;
use crate::func::paths::LiteralSet;
use crate::func::variables::*;

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

#[derive(Copy, Clone, PartialEq)]
pub enum Operator {
    AND,
    OR,
    NAND,
    NOR,
}

#[derive(Clone, PartialEq)]
pub struct Children {
    pub data: Rc<Vec<Expr>>,
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
    pub fn join(self, iter: &mut Iterator<Item = Expr>) -> Expr {
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
        let ret = match self.negate(neg) {
            Operator::AND => (Operator::AND, false),
            Operator::OR => (Operator::OR, false),
            Operator::NAND => (Operator::OR, true),
            Operator::NOR => (Operator::AND, true),
        };
        //        println!("PROPAGATE: {},{},{} => {},{}", self, self.is_neg(),neg, ret.0, ret.1);
        ret
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
}

/*  ******************** Manipulate the list of children ******************** */
impl Children {
    pub fn from_expressions(iter: &mut Iterator<Item = Expr>) -> Children {
        Children {
            data: Rc::new(iter.collect()),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
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

        let disjunction = op.is_disjunction();
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
    pub fn replace_literal(&self, f: &Fn(usize, bool) -> Option<Expr>) -> Option<Expr> {
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

    pub fn get_literals(&self) -> LiteralSet {
        self._get_literals(false)
    }

    pub fn _get_literals(&self, neg: bool) -> LiteralSet {
        match self {
            Expr::TRUE => LiteralSet::new(),
            Expr::FALSE => LiteralSet::new(),
            Expr::ATOM(u) => LiteralSet::with(*u, neg),
            Expr::NATOM(u) => LiteralSet::with(*u, !neg),
            Expr::OPER(o, c) => c.get_literals(o.is_neg() != neg),
        }
    }
}

impl Children {
    fn replace_literal(&self, f: &Fn(usize, bool) -> Option<Expr>, op: Operator) -> Option<Expr> {
        let children: Vec<Option<Expr>> = self.data.iter().map(|c| c.replace_literal(f)).collect();
        let count = children.iter().filter(|c| c.is_some()).count();
        if count > 1 {
            let children = self.data.iter().zip(children.into_iter()).map(|(c, r)| {
                if r.is_some() {
                    r.unwrap()
                } else {
                    c.clone()
                }
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

    // TODO: cache the result?
    pub fn get_literals(&self, neg: bool) -> LiteralSet {
        let mut used = LiteralSet::new();
        for child in self.data.iter() {
            let u = child._get_literals(neg);
            used.union_with(&u);
        }
        used
    }
}

/* ************************************************************************************* */
/* ************************************* Formatting ************************************ */
/* ************************************************************************************* */

struct FormatContext<'a> {
    parent_priority: u8,
    group: Option<&'a Group>,
}

impl<'a> FormatContext<'a> {
    fn new() -> FormatContext<'a> {
        FormatContext {
            parent_priority: 0,
            group: None,
        }
    }

    fn from_group(grp: &Group) -> FormatContext {
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
            None => write!(f, "{}", uid),
            Some(g) => write!(f, "{}", g.get_name(uid)),
        }
    }
}

impl func::Grouped for Expr {
    fn gfmt(&self, group: &Group, f: &mut fmt::Formatter) -> fmt::Result {
        let mut context = FormatContext::from_group(group);
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
    use super::*;
    use crate::func::variables::*;

    #[test]
    fn conj_extension() {
        let grp = Group::new();
        let a = Expr::ATOM(Group::get_var_from_name(&grp, "A"));
        let b = Expr::ATOM(Group::get_var_from_name(&grp, "B"));
        let c = Expr::ATOM(Group::get_var_from_name(&grp, "C"));

        let x = a & b | c;
        // let y = a & c;

        // assert_eq!(format!("{}",y), "");

        let s = x.simplify();
        assert_eq!(s, None);
        assert_eq!(4, 4);
    }
}
