//! Logical model: collections of components, with associated variables and functions

use std::fmt;

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
    fn get_component(&self, name: &str) -> Option<usize>;

    /// Find a variable based on the name of the component and the threshold value
    fn get_variable(&self, name: &str, value: usize) -> Option<usize> {
        let base_component = self.get_component(name);
        if let Some(uid) = base_component {
            return self.get_associated_variable(uid, value);
        }
        None
    }

    /// Find a variable for an existing component and a threshold value
    fn get_associated_variable(&self, cid: usize, value: usize) -> Option<usize>;

    /// Find or create a component with a given name
    fn ensure_component(&mut self, name: &str) -> usize;

    /// Find or create a variable with a given component name and threshold value
    fn ensure_variable(&mut self, name: &str, value: usize) -> usize {
        let cid = self.ensure_component(name);
        self.ensure_associated_variable(cid, value)
    }

    /// Find or create a variable for an existing component and a specific threshold value
    fn ensure_associated_variable(&mut self, cid: usize, value: usize) -> usize;

    /// Assign a Boolean condition for a specific threshold
    fn set_rule(&mut self, target: usize, value: usize, rule: Formula);

    fn lock(&mut self, uid: usize, value: bool) {
        self.set_rule(uid, 1, Formula::from(Expr::from_bool(value)));
    }

    fn get_name(&self, uid: usize) -> &str;

    fn set_name(&mut self, uid: usize, name: String) -> bool;

    /// Rename a component.
    /// Returns false if the new name is invalid or already assigned
    /// to another component
    fn rename(&mut self, source: &str, name: String) -> bool {
        match self.get_component(source) {
            None => false,
            Some(u) => self.set_name(u, name),
        }
    }

    fn variables<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Variable)> + 'a>;

    fn components<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Component)> + 'a>;

    fn rule(&self, uid: usize) -> &DynamicRule;

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
    rule: DynamicRule,
    variables: HashMap<usize, usize>,
}

pub struct DynamicRule {
    assignments: Vec<Assign>,
}

/// A formula associated with a target value
pub struct Assign {
    pub target: usize,
    pub formula: Formula,
}

impl Component {
    fn new(name: String) -> Self {
        Component {
            name: name,
            rule: DynamicRule::new(),
            variables: HashMap::new(),
        }
    }
}

impl Variable {
    fn new(component: usize, value: usize) -> Self {
        Variable {
            component: component,
            value: value,
        }
    }
}

impl DynamicRule {
    fn new() -> Self {
        DynamicRule {
            assignments: vec![],
        }
    }

    pub fn extend<T: BoolRepr>(&mut self, value: usize, condition: T) {
        self.extend_formula(value, Formula::from(condition));
    }

    pub fn extend_formula(&mut self, value: usize, condition: Formula) {
        self.assignments.insert(
            self.assignments.len(),
            Assign {
                target: value,
                formula: condition,
            },
        )
    }

    fn set_expression<T: BoolRepr>(&mut self, condition: T, v: usize) {
        self.assignments.clear();
        self.extend(v, condition);
    }

    fn set_formula(&mut self, f: Formula, v: usize) {
        self.assignments.clear();
        self.extend_formula(v, f);
    }

    pub fn as_func<T: FromBoolRepr>(&self) -> T {
        if self.assignments.len() < 1 {
            return Expr::FALSE.into_repr().convert_as();
        }

        // FIXME: build the expr for target value 1
        self.assignments.get(0).unwrap().convert()
    }
}

impl Assign {
    pub fn convert<T: FromBoolRepr>(&self) -> T {
        self.formula.convert_as()
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- {}", self.target, self.formula)
    }
}
