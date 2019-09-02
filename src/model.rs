//! Logical model: collections of components, with associated variables and functions

use std::collections::HashMap;
use std::fmt;

use regex::Regex;
use slab;

use crate::func::expr::*;
use crate::func::*;
use std::fmt::Display;

pub mod actions;
pub mod io;
pub mod modifier;

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

    /// Retrieve a component by name
    fn get_component(&self, name: &str) -> Option<usize>;

    /// Retrieve a variable by name and value
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

    fn get_associated_variable(&self, uid: usize, value: usize) -> Option<usize>;

    /// Retrieve or create a named component
    fn ensure_component(&mut self, name: &str) -> usize;

    fn ensure_variable(&mut self, name: &str, value: usize) -> usize {
        let uid = self.ensure_component(name);
        if value == 1 {
            return uid;
        }
        self.ensure_associated_variable(uid, value)
    }

    fn ensure_associated_variable(&mut self, uid: usize, value: usize) -> usize;

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

    fn variables<'a>(&'a self) -> &'a Vec<usize>;

    fn rule<'a>(&'a self, uid: usize) -> &'a DynamicRule;

    fn format_name(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result;

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;

    fn as_namer(&self) -> &dyn VariableNamer;
    fn for_display(&self) -> &dyn Display;
}

impl<T> VariableNamer for T where T: QModel {
    fn format_name(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result {
        QModel::format_name(self, f, uid)
    }
}

pub type LQModelRef = Box<dyn QModel>;
pub fn new_model() -> LQModelRef {
    Box::new(LQModel::new())
}

struct Variable {
    uid: usize,
    next: usize,
    info: VariableInfo,
}

enum VariableInfo {
    COMPONENT(Component),
    EXTENDED(Extension),
    NEGATION(Extension),
}

struct Component {
    name: String,
    rule: DynamicRule,
}

struct Extension {
    component: usize,
    value: usize,
}

pub struct DynamicRule {
    assignments: Vec<Assign>,
}

/// A formula associated with a target value
pub struct Assign {
    pub target: usize,
    pub formula: Formula,
}

struct LQModel {
    components: slab::Slab<Variable>,
    name2uid: HashMap<String, usize>,
    var_indices: Vec<usize>,
}

impl VariableInfo {

    fn set_name(&mut self, name: &str) {
        // FIXME
    }

    fn set_rule(&mut self, value: usize, rule: Formula) {

        match self {
            VariableInfo::COMPONENT(c) => c.rule.set_formula(rule, value),
            _ => panic!("set_rule should only be applied on components"),
        }
        // FIXME
    }
}

impl DynamicRule {
    fn new() -> Self {
        DynamicRule {
            assignments: vec!(),
        }
    }
}

impl QModel for LQModel {

    fn get_component(&self, name: &str) -> Option<usize> {
        if let Some(uid) = self.name2uid.get(name) {
            return Some(*uid);
        }
        None
    }

    fn get_associated_variable(&self, uid: usize, value: usize) -> Option<usize> {
        // FIXME
        if value < 2 || value > 9 {
            if value != 1 {
                eprintln!("Tried to access a strange value: {}", value);
            }
            return Some(uid);
        }

        let mut curid = self.components[uid].next;
        while curid > 0 {
            let var = &self.components[curid];
            if let VariableInfo::EXTENDED(info) = &var.info {
                if info.value == value {
                    return Some(curid);
                }
            }
            curid = var.next;
        }
        None
    }

    fn ensure_component(&mut self, name: &str) -> usize {
        if let Some(uid) = self.name2uid.get(name) {
            return *uid;
        }

        let cmps = &mut self.components;
        let entry = self.components.vacant_entry();
        let uid = entry.key();
        entry.insert(Variable {
            uid: uid,
            next: 0,
            info: VariableInfo::COMPONENT(Component::new(String::from(name))),
        });
        uid
    }

    fn ensure_associated_variable(&mut self, uid: usize, value: usize) -> usize {
        if value < 2 || value > 9 {
            if value != 1 {
                eprintln!("Tried to access a strange value: {}", value);
            }
            return uid;
        }

        // Search an existing

        // FIXME
        let entry = self.components.vacant_entry();
        let vid = entry.key();
        let info = Extension {
            component: uid,
            value: value,
        };
        entry.insert(Variable {
            uid: uid,
            next: 0,
            info: VariableInfo::EXTENDED(info),
        });
        uid
    }

    fn set_rule(&mut self, target: usize, value: usize, rule: Formula) {
        let info = &mut self.components[target].info;
        match info {
            VariableInfo::COMPONENT(c) => c.rule.set_formula(rule, value),
            VariableInfo::EXTENDED(e) =>  {
                let uid = e.component;
                self.set_rule(uid, value, rule)
            },
            VariableInfo::NEGATION(e) =>  {
                let uid = e.component;
                self.set_rule(uid, value, rule)
            },
        }
    }

    fn get_name(&self, uid: usize) -> &str {
        let var = &self.components[uid];
        match &var.info {
            VariableInfo::COMPONENT(c) => &c.name,
            VariableInfo::EXTENDED(e) => self.get_name(e.component),
            VariableInfo::NEGATION(e) => self.get_name(e.component),
        }
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

        let old_name = self.get_name(uid).to_string();
        self.name2uid.remove(&old_name);
        self.name2uid.insert(name.clone(), uid);

// FIXME: implement set_name

        let info = &mut self.components[uid].info;
        match info {
            VariableInfo::COMPONENT(c) => {
                c.name = name;
                true
            },
            VariableInfo::EXTENDED(e) => {
                let cid = e.component;
                self.set_name(cid, name)
            },
            VariableInfo::NEGATION(e) => {
                let cid = e.component;
                self.set_name(cid, name)
            },
        }
    }

    fn variables<'a>(&'a self) -> &'a Vec<usize> {
        &self.var_indices
    }

    fn rule<'a>(&'a self, uid: usize) -> &'a DynamicRule {
        let var = &self.components[uid];
        match &var.info {
            VariableInfo::COMPONENT(c) => &c.rule,
            VariableInfo::EXTENDED(e) => self.rule(e.component),
            VariableInfo::NEGATION(e) => self.rule(e.component),
        }
    }


    fn format_name(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result {
        let var = &self.components[uid];
        match &var.info {
            VariableInfo::COMPONENT(c) => write!(f, "{}", c.name),
            VariableInfo::EXTENDED(e) => write!(f, ":{}", e.value),
            VariableInfo::NEGATION(e) => write!(f, "^{}", e.value),

        }
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (_, component) in &self.components {
            match &component.info {
                VariableInfo::COMPONENT(c) => {
                    for a in &c.rule.assignments {
                        write!(f, "{}", c.name)?;
                        if a.target != 1 {
                            write!(f, ":{}", a.target)?;
                        }
                        write!(f, " <- ")?;
// FIXME                        a.formula.gfmt(self, f)?;
                        writeln!(f)?;
                    }
                },
                _ => (),
            }
        }
        write!(f, "")
    }

    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }
    fn for_display(&self) -> &dyn Display {
        self
    }
}

impl Component {
    fn new(name: String) -> Self {
        Component {
            name: name,
            rule: DynamicRule::new(),
        }
    }
}

impl DynamicRule {

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

impl LQModel {
    pub fn new() -> LQModel {
        LQModel {
            components: slab::Slab::new(),
            name2uid: HashMap::new(),
            var_indices: vec!(),
        }
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


impl fmt::Display for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        QModel::fmt(self, f)
    }
}

impl fmt::Debug for LQModel {
    fn fmt(&self, ft: &mut fmt::Formatter) -> fmt::Result {
        write!(ft, "{}", self)
    }
}
