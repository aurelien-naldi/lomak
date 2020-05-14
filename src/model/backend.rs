//! Concrete implementation for the QModel trait

use std::collections::HashMap;
use std::fmt;

use slab;

use crate::func::{Formula, Grouped, VariableNamer};
use crate::model::*;

pub fn new_model() -> impl QModel {
    LQModel {
        components: slab::Slab::new(),
        variables: slab::Slab::new(),
        name2uid: HashMap::new(),
    }
}

struct LQModel {
    components: slab::Slab<SharedComponent>,
    variables: slab::Slab<Variable>,
    name2uid: HashMap<String, usize>,
}

impl LQModel {
    fn component_mut<'a>(&'a mut self, cid: usize) -> SharedComponent {
        self.components[cid].clone()
    }
}

impl QModel for LQModel {
    fn component_by_name(&self, name: &str) -> Option<usize> {
        if let Some(uid) = self.name2uid.get(name) {
            return Some(*uid);
        }
        None
    }

    fn associated_variable_with_threshold(&self, cid: usize, value: usize) -> Option<usize> {
        if value < 1 || value > 9 {
            eprintln!("Tried to access a strange value: {}", value);
            return self.associated_variable(cid);
        }

        if let Some(vid) = self.get_component_ref(cid).borrow().variables().get(&value) {
            return Some(*vid);
        }
        None
    }

    fn ensure_component(&mut self, name: &str) -> usize {
        if let Some(uid) = self.component_by_name(name) {
            return uid;
        }

        // Create a new component
        // TODO: maintain a list of components?
        let n = name.to_owned();
        let cid = self
            .components
            .insert(Component::new(n.clone()).into_shared());
        self.name2uid.insert(n, cid);
        cid
    }

    fn ensure_associated_variable(&mut self, cid: usize, value: usize) -> usize {
        if value < 1 || value > 9 {
            eprintln!("Tried to access a strange value: {}", value);
            return self.ensure_associated_variable(cid, 1);
        }

        // Return existing variable
        if let Some(vid) = self.associated_variable_with_threshold(cid, value) {
            return vid;
        }

        // Create a new variable and add it to the component
        let component = self.component_mut(cid);
        let vid = self.variables.insert(Variable::new(cid, value));
        component
            .rc
            .as_ref()
            .borrow_mut()
            .variables
            .insert(value, vid);
        vid
    }

    fn set_rule(&mut self, target: usize, rule: Formula) {
        let var = &self.variables[target];
        let cid = var.component;
        let val = var.value;
        let cpt = self.component_mut(cid);
        cpt.borrow_mut().set_formula(rule, val);
    }

    fn extend_rule(&mut self, target: usize, rule: Formula) {
        let var = &self.variables[target];
        let cid = var.component;
        let val = var.value;
        let cpt = self.component_mut(cid);
        cpt.borrow_mut().extend_formula(val, rule);
    }

    fn set_component_name(&mut self, uid: usize, name: String) -> bool {
        // Reject invalid new names
        if !RE_UID.is_match(&name) {
            return false;
        }

        // Reject existing names
        if let Some(u) = self.name2uid.get(&name) {
            return *u == uid;
        }

        let old_name = self.get_name(uid).to_string();
        self.name2uid.remove(&old_name);
        self.name2uid.insert(name.clone(), uid);

        let cpt = self.component_mut(uid);
        cpt.borrow_mut().set_name(name);
        true
    }

    fn get_variable(&self, var: usize) -> Variable {
        *self.variables.get(var).unwrap()
    }

    fn variables<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a Variable)> + 'a> {
        Box::new(self.variables.iter())
    }

    fn components<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, &'a SharedComponent)> + 'a> {
        Box::new(self.components.iter())
    }

    fn get_component_ref(&self, uid: usize) -> SharedComponent {
        self.components[uid].clone()
    }
}

impl LQModel {}

impl VariableNamer for LQModel {
    fn format_name(&self, f: &mut fmt::Formatter, uid: usize) -> fmt::Result {
        let var = &self.variables[uid];
        let cmp = self.get_component_ref(var.component);
        let cmp = cmp.borrow();
        if var.value != 1 {
            write!(f, "{}:{}", cmp.name, var.value)
        } else {
            write!(f, "{}", cmp.name)
        }
    }

    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }
}

impl fmt::Display for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let namer = self.as_namer();

        for (_, component) in self.components() {
            let component = component.borrow();
            for a in component.assignments() {
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
        for (u, c) in self.components() {
            let c = c.borrow();
            write!(f, "{} ({}):", u, c.name)?;
            for v in c.variables().keys() {
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
