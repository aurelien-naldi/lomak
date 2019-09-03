//! Logical model: collections of components, with associated variables and functions

use std::fmt;

use regex::Regex;

use crate::func::expr::*;
use crate::func::*;
use std::fmt::Display;
use std::collections::HashMap;

pub mod actions;
pub mod io;
pub mod modifier;

mod backend;

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
    static ref RE_UID: Regex = Regex::new(r"[a-zA-Z][a-zA-Z01-9_]*").unwrap();
}

/// Define the public API for logical models
///
/// A model has a list of Boolean variables, which can be Boolean or
/// multivalued components forming groups of related Boolean variables.
///
/// Each component has a name, associated variables also have a value.
///
/// Finally, each component is associated to a function defining the condition
/// required for its activation
pub trait QModel {

    /// Find a component by name if it exists
    fn get_component(&self, name: &str) -> Option<usize>;

    /// Find a variable based on the name of the component and the threshold value
    fn get_variable(&self, name: &str, value: usize) -> Option<usize> {
        let base_component = self.get_component(name);
        if value == 1 {
            return base_component;
        }
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

    fn set_rule(&mut self, target: usize, value: usize, rule: Formula);

    fn lock(&mut self, uid: usize, value: bool) {
        self.set_rule(uid, 1, Formula::from(Expr::from_bool(value)));
    }

    fn knockout(&mut self, uid: usize) {
        self.lock(uid, false);
    }

    fn knockin(&mut self, uid: usize) {
        self.lock(uid, true);
    }

    fn perturbation(&mut self, cmd: &str) {
        match RE_PRT.captures(cmd) {
            None => println!("Invalid perturbation parameter: {}", cmd),
            Some(cap) => {
                if let Some(uid) = self.get_component(&cap[1]) {
                    match &cap[2] {
                        "0" => return self.knockout(uid),
                        "1" => return self.knockin(uid),
                        _ => {
                            println!("Invalid target value: {}", &cap[2]);
                            ()
                        }
                    }
                }
            }
        }
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

    fn variables(&self) -> &Vec<usize>;

    fn rule(&self, uid: usize) -> &DynamicRule;

    fn as_namer(&self) -> &dyn VariableNamer;

    fn for_display(&self) -> &dyn Display;
}

pub type LQModelRef = Box<dyn QModel>;
pub fn new_model() -> LQModelRef {
    Box::new(backend::new_model())
}

pub struct Variable {
    component: usize,
    value: usize,
}

pub struct Component {
    name: String,
    rule: DynamicRule,
    variables: HashMap<usize,usize>,
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
        Variable{
            component: component,
            value: value,
        }
    }
}

impl DynamicRule {

    fn new() -> Self {
        DynamicRule {
            assignments: vec!(),
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
