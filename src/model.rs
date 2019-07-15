//! Logical model: collections of components, with associated variables and functions

use regex::Regex;
use slab;
use std::collections::HashMap;
use std::fmt;

use crate::func::expr::*;
use crate::func::variables::VariableNamer;
use crate::func::Grouped;
use crate::func::*;

pub mod actions;
pub mod io;
pub mod modifier;

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
    static ref RE_UID: Regex = Regex::new(r"[a-zA-Z][a-zA-Z01-9_]*").unwrap();
}

pub struct Component {
    name: String,
    uid: usize,
    variables: Vec<usize>,
    assignments: Vec<Assign>,
}

struct Variable {
    component: usize,
    value: usize,
}

/// A formula associated with a target value
pub struct Assign {
    pub target: usize,
    pub formula: Formula,
}

pub struct LQModel {
    components: slab::Slab<Component>,
    variables: slab::Slab<Variable>,
    name2uid: HashMap<String, usize>,
}

impl Component {
    // Create a new component and insert it in the model
    fn create(model: &mut LQModel, name: String) -> usize {
        let entry = model.components.vacant_entry();
        let key = entry.key();
        entry.insert(Component {
            uid: key,
            name: name,
            variables: vec![],
            assignments: vec![],
        });
        key
    }

    pub fn get_variable(&self, value: usize) -> Option<usize> {
        if value < 1 || value > self.variables.len() {
            return None;
        }
        Some(self.variables[value - 1])
    }

    pub fn ensure_variable(&mut self, model: &mut LQModel, value: usize) -> usize {
        // Make sure that we stay in a valid range of values
        if value < 1 || value > 9 {
            return self.ensure_variable(model, 1);
        }

        for v in self.variables.len()..value {
            self.variables.push(model.variables.insert(Variable {
                component: self.uid,
                value: v + 1,
            }));
        }

        self.variables[value - 1]
    }

    pub fn extend<T: BoolRepr>(&mut self, value: usize, condition: T) {
        self.assignments.insert(
            self.assignments.len(),
            Assign {
                target: value,
                formula: Formula::from(condition),
            },
        )
    }

    pub fn set_expression<T: BoolRepr>(&mut self, value: T, v: usize) {
        self.set_formula(Formula::from(value), v);
    }

    pub fn set_formula(&mut self, f: Formula, v: usize) {
        self.assignments.clear();
        self.assignments.insert(
            0,
            Assign {
                target: v,
                formula: f,
            },
        );
    }

    pub fn as_func<T: FromBoolRepr>(&self) -> T {
        if self.assignments.len() < 1 {
            return Expr::FALSE.into_repr().convert_as();
        }

        // FIXME: build the expr for target value 1
        self.assignments.get(0).unwrap().convert()
    }
}

impl LQModel {
    pub fn new() -> LQModel {
        LQModel {
            components: slab::Slab::new(),
            variables: slab::Slab::new(),
            name2uid: HashMap::new(),
        }
    }

    pub fn set_rule(&mut self, target: usize, value: usize, rule: Formula) {
        self.components[target].set_formula(rule, value);
    }

    pub fn lock(mut self, uid: usize, value: bool) -> Self {
        self.set_rule(uid, 1, Formula::from(Expr::from_bool(value)));
        self
    }

    pub fn knockout(self, uid: usize) -> Self {
        self.lock(uid, false)
    }

    pub fn knockin(self, uid: usize) -> Self {
        self.lock(uid, true)
    }

    pub fn perturbation(self, cmd: &str) -> Self {
        match RE_PRT.captures(cmd) {
            None => println!("Invalid perturbation parameter: {}", cmd),
            Some(cap) => {
                if let Some(uid) = self.node_id(&cap[1]) {
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
        self
    }

    pub fn components(&self) -> &slab::Slab<Component> {
        &self.components
    }
}

impl VariableNamer for LQModel {
    fn node_id(&self, name: &str) -> Option<usize> {
        match self.name2uid.get(name) {
            Some(uid) => Some(*uid),
            None => None,
        }
    }

    fn get_node_id(&mut self, name: &str) -> Option<usize> {
        match self.name2uid.get(name) {
            Some(uid) => return Some(*uid),
            None => (),
        };

        if !RE_UID.is_match(&name) {
            return None;
        }

        let name = String::from(name);
        let uid = Component::create(self, name.clone());
        self.name2uid.insert(name, uid);
        Some(uid)
    }

    fn get_name(&self, uid: usize) -> String {
        self.components[uid].name.clone()
    }

    fn set_name(&mut self, uid: usize, name: String) -> bool {
        // Reject invalid new names
        if !RE_UID.is_match(&name) {
            return false;
        }

        // Reject existing names
        match self.name2uid.get(&name) {
            Some(u) => return *u == uid,
            None => (),
        }

        self.name2uid.remove(&self.get_name(uid));
        self.name2uid.insert(name.clone(), uid);
        self.components[uid].name = name;

        true
    }
}

impl fmt::Display for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (_, component) in &self.components {
            writeln!(f, "{}", component)?;
        }
        write!(f, "")
    }
}

impl fmt::Debug for LQModel {
    fn fmt(&self, ft: &mut fmt::Formatter) -> fmt::Result {
        write!(ft, "{}", self)
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for a in &self.assignments {
            writeln!(f, "{}: {}", self.name, a)?;
        }
        write!(f, "")
    }
}

impl Grouped for Component {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", self.name)?;
        for a in &self.assignments {
            a.gfmt(namer, f)?;
        }
        write!(f, "")
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

impl Grouped for Assign {
    fn gfmt(&self, namer: &dyn variables::VariableNamer, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- ", self.target)?;
        self.formula.gfmt(namer, f)
    }
}
