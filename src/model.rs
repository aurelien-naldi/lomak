//! Logical model: collections of components, with associated variables and functions

use std::fmt;
use std::rc::Rc;

use regex::Regex;

use crate::func::expr::*;
use crate::func::*;
use std::collections::HashMap;
use std::fmt::Display;

pub mod actions;
pub mod io;
pub mod modifier;

mod backend;

lazy_static! {
    static ref RE_UID: Regex = Regex::new(r"[a-zA-Z][a-zA-Z01-9_]*").unwrap();
}

/// Public API for qualitative models
///
/// A model contains a list of named components, which are associated to
/// one or several Boolean variables for each qualitative threshold.
/// Components and variables are identified by unique handles (positive integers).
///
/// Finally, each component is associated to a list of Boolean functions defining
/// the conditions required for the activation of each threshold.
pub trait QModel: VariableNamer {
    /// Find a component by name if it exists
    /// Components are NOT valid variables: they carry the name and
    /// the list of proper Boolean variables for each threshold.
    fn component_by_name(&self, name: &str) -> Option<usize>;

    /// Find the base variable corresponding to the name of a component
    ///
    /// Equivalent to variable_by_name_and_threshold(name, 1)
    ///
    /// TODO: should it attempt to extract the threshold from the name's suffix?
    fn variable_by_name(&self, name: &str) -> Option<usize> {
        self.variable_by_name_and_threshold(name, 1)
    }

    /// Find a variable based on the name of the component and the threshold value
    fn variable_by_name_and_threshold(&self, name: &str, value: usize) -> Option<usize> {
        let base_component = self.component_by_name(name);
        if let Some(uid) = base_component {
            return self.associated_variable_with_threshold(uid, value);
        }
        None
    }

    /// Find the base variable for an existing component
    ///
    /// Equivalent to associated_variable_with_threshold(cid, 1)
    fn associated_variable(&self, cid: usize) -> Option<usize> {
        self.associated_variable_with_threshold(cid, 1)
    }

    /// Find a variable for an existing component and a threshold value
    fn associated_variable_with_threshold(&self, cid: usize, value: usize) -> Option<usize>;

    /// Find or create a component with a given name
    fn ensure_component(&mut self, name: &str) -> usize;

    /// Find or create a variable with a given component name and threshold value
    ///
    /// Equivalent to ensure_variable_with_threshold(name, 1)
    fn ensure_variable(&mut self, name: &str) -> usize {
        self.ensure_variable_with_threshold(name, 1)
    }

    /// Find or create a variable with a given component name and threshold value
    fn ensure_variable_with_threshold(&mut self, name: &str, value: usize) -> usize {
        let cid = self.ensure_component(name);
        self.ensure_associated_variable(cid, value)
    }

    /// Find or create a variable for an existing component and a specific threshold value
    fn ensure_associated_variable(&mut self, cid: usize, value: usize) -> usize;

    /// Assign a Boolean condition to a variable
    fn set_rule(&mut self, variable: usize, rule: Formula);

    /// Assign a Boolean condition for a specific threshold
    fn extend_rule(&mut self, variable: usize, rule: Formula);

    fn lock(&mut self, uid: usize, value: bool) {
        self.set_rule(uid, Formula::from_bool(value));
    }

    fn get_name(&self, uid: usize) -> &str;

    fn set_component_name(&mut self, uid: usize, name: String) -> bool;

    /// Rename a component.
    /// Returns false if the new name is invalid or already assigned
    /// to another component
    fn rename(&mut self, source: &str, name: String) -> bool {
        match self.component_by_name(source) {
            None => false,
            Some(u) => self.set_component_name(u, name),
        }
    }

    fn variables<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Variable)> + 'a>;

    fn components<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Component)> + 'a>;

    fn get_component<'a>(&'a self, uid: usize) -> &'a Component;

    fn for_display(&self) -> &dyn Display;
}

pub type LQModelRef = Box<dyn QModel>;
pub fn new_model() -> LQModelRef {
    Box::new(backend::new_model())
}

/// A Boolean variable associated to a qualitative threshold of one of the components
pub struct Variable {
    component: usize,
    value: usize,
}

/// The component of a model provide the name, a list of
/// available variables and the dynamic rule.
pub struct Component {
    name: String,
    variables: HashMap<usize, usize>,
    assignments: Vec<Assign>,
    cached_rules: HashMap<usize, Formula>,
}

/// A formula associated with a target value
pub struct Assign {
    pub target: usize,
    pub formula: Formula,
}

impl Component {
    fn new(name: String) -> Self {
        Component {
            name,
            variables: HashMap::new(),
            assignments: vec![],
            cached_rules: HashMap::new(),
        }
    }

    fn get_formula(&self, value: usize) -> Expr {
        let mut expr = Expr::FALSE;
        for asg in self.assignments.iter() {
            let cur: Rc<Expr> = asg.formula.convert_as();
            if asg.target < value {
                expr = expr.and(&cur.not());
            } else {
                expr = expr.or(&cur);
            }
        }


        match self.variables.get( &(value+1)) {
            None => (),
            Some(next_var) => {
                let cur_var = self.variables.get( &value).unwrap();
                let cur_active = Expr::ATOM(*cur_var);
                let next_active = Expr::ATOM(*next_var);
                expr = expr.or( &cur_active.and(&next_active));
            }
        }

        if value > 1 {
            let prev_var = self.variables.get( &(value-1)).unwrap();
            let prev_active = Expr::ATOM(*prev_var);
            expr = expr.and( &prev_active);
        }

        expr.simplify().unwrap_or(expr)
    }

    pub fn extend<T: BoolRepr>(&mut self, value: usize, condition: T) {
        self.extend_formula(value, Formula::from(condition));
    }

    pub fn extend_formula(&mut self, value: usize, condition: Formula) {
        if !self.variables.contains_key(&value) {
            eprintln!("ERROR: Can not assign a non-existing variable -> using default threshold");
            return self.extend_formula(1, condition);
        }
        self.cached_rules.clear();
        self.assignments.push(Assign {
            target: value,
            formula: condition,
        })
    }

    fn set_formula(&mut self, f: Formula, v: usize) {
        self.assignments.clear();
        self.extend_formula(v, f);
    }

    pub fn as_func<T: FromBoolRepr>(&self, value: usize) -> Rc<T> {
        let expr = self.get_formula(value).into_repr();
        let expr: Rc<T> = T::convert(&expr);
        expr
    }
}

impl Variable {
    fn new(component: usize, value: usize) -> Self {
        Variable { component, value }
    }
}

impl Assign {
    pub fn convert<T: FromBoolRepr>(&self) -> Rc<T> {
        self.formula.convert_as()
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- {}", self.target, self.formula)
    }
}
