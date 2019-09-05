//! Concrete implementation for the QModel trait

use std::collections::HashMap;
use std::fmt;

use slab;

use crate::func::{Grouped, VariableNamer};
use crate::model::*;
use std::fmt::Display;

pub fn new_model() -> impl QModel {
    LQModel {
        components: slab::Slab::new(),
        variables: slab::Slab::new(),
        name2uid: HashMap::new(),
    }
}

struct LQModel {
    components: slab::Slab<Component>,
    variables: slab::Slab<Variable>,
    name2uid: HashMap<String, usize>,
}

impl QModel for LQModel {
    fn get_component(&self, name: &str) -> Option<usize> {
        if let Some(uid) = self.name2uid.get(name) {
            return Some(*uid);
        }
        None
    }

    fn get_associated_variable(&self, cid: usize, value: usize) -> Option<usize> {
        if value < 1 || value > 9 {
            eprintln!("Tried to access a strange value: {}", value);
            return self.get_associated_variable(cid, 1);
        }

        if let Some(vid) = self.components[cid].variables.get(&value) {
            return Some(*vid);
        }
        None
    }

    fn ensure_component(&mut self, name: &str) -> usize {
        if let Some(uid) = self.get_component(name) {
            return uid;
        }

        // Create a new component
        // TODO: maintain a list of components?
        let n = name.to_owned();
        let cid = self.components.insert(Component::new(n.clone()));
        self.name2uid.insert(n, cid);
        cid
    }

    fn ensure_associated_variable(&mut self, cid: usize, value: usize) -> usize {
        if value < 1 || value > 9 {
            eprintln!("Tried to access a strange value: {}", value);
            return self.ensure_associated_variable(cid, 1);
        }

        // Return existing variable
        if let Some(vid) = self.get_associated_variable(cid, value) {
            return vid;
        }

        // Create a new variable and add it to the component
        let component = &mut self.components[cid];
        let vid = self.variables.insert(Variable::new(cid, value));
        component.variables.insert(value, vid);

        return vid;
    }

    fn set_rule(&mut self, target: usize, value: usize, rule: Formula) {
        &self.components[target].rule.set_formula(rule, value);
    }

    fn get_name(&self, uid: usize) -> &str {
        &self.components[uid].name
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

        self.components[uid].name = name;
        true
    }

    fn variables<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Variable)> + 'a> {
        Box::new(self.variables.iter())
    }

    fn components<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Component)> + 'a> {
        Box::new(self.components.iter())
    }

    fn rule(&self, uid: usize) -> &DynamicRule {
        &self.components[uid].rule
    }

    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }

    fn for_display(&self) -> &dyn Display {
        self
    }
}

impl LQModel {}

impl VariableNamer for LQModel {
    fn format_name(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result {
        let var = &self.variables[uid];
        let cmp = &self.components[var.component];
        if var.value != 1 {
            write!(f, "{}:{}", cmp.name, var.value)
        } else {
            write!(f, "{}", cmp.name)
        }
    }
}

impl fmt::Display for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let namer = self.as_namer();

        for (_, component) in &self.components {
            for a in &component.rule.assignments {
                write!(f, "{}", component.name)?;
                if a.target != 1 {
                    write!(f, ":{}", a.target)?;
                }
                write!(f, " <- ")?;
                a.formula.gfmt(namer, f)?;
                writeln!(f)?;
            }
        }
        write!(f, "")
    }
}

impl fmt::Debug for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (u, c) in &self.components {
            write!(f, "{} ({}):", u, c.name)?;
            for (v, _) in &c.variables {
                write!(f, "  {}", v)?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;

        for (u, v) in &self.variables {
            writeln!(f, "{}: {}:{}", u, v.component, v.value)?;
        }
        writeln!(f)?;

        write!(f, "{}", self)
    }
}
