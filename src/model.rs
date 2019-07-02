//! Logical model: collections of functions

use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use slab;

use crate::func::expr::Expr;
use crate::func::{variables, Formula};
use crate::func::variables::VariableNamer;
use crate::func::Grouped;

pub mod actions;
pub mod io;
pub mod modifier;
pub mod rule;

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
}

pub struct Component {
    name: String,
    rule: rule::Rule,
}

pub struct LQModel {
    grp: variables::Group,
    components: slab::Slab<Component>,
    name2uid: HashMap<String, usize>,
}

impl LQModel {
    pub fn new() -> LQModel {
        LQModel {
            grp: variables::Group::new(),
            components: slab::Slab::new(),
            name2uid: HashMap::new(),
        }
    }

    pub fn set_rule(&mut self, target: usize, value: u8, rule: Formula) {
        if let Some(f) = self.rules[target] {
            f.set_formula(rule);
            return;
        }
        self.rules.insert(target, rule::Rule::from_formula(rule));
    }

    pub fn lock(mut self, uid: usize, value: bool) -> Self {
        self.set_rule(uid, 1, Formula::from( Expr::from_bool(value) ) );
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

    pub fn rules(&self) -> &HashMap<usize, rule::Rule> {
        &self.rules
    }
}

/// Delegate the VariableNamer trait to the internal Group
//impl VariableNamer for LQModel {
//    fn node_id(&self, name: &str) -> Option<usize> {
//        self.grp.node_id(name)
//    }
//
//    fn get_node_id(&mut self, name: &str) -> Option<usize> {
//        self.grp.get_node_id(name)
//    }
//
//    fn get_name(&self, uid: usize) -> String {
//        self.grp.get_name(uid)
//    }
//
//    fn set_name(&mut self, uid: usize, name: String) -> bool {
//        self.grp.set_name(uid, name)
//    }
//
//}


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

        if !variables::RE_UID.is_match(&name) {
            return None;
        }

        let name = String::from(name);
        let uid = self.cur_uid;
        self.cur_uid += 1;
        let ret = uid;
        self.name2uid.insert(name.clone(), uid);
        self.uid2name.insert(uid, name);
        Some(ret)
    }

    fn get_name(&self, uid: usize) -> String {
        match self.uid2name.get(&uid) {
            Some(name) => name.clone(),
            None => format!("_{}", uid),
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

        self.name2uid.remove(&self.get_name(uid));
        self.name2uid.insert(name.clone(), uid);
        self.uid2name.insert(uid, name);

        true
    }
}

impl fmt::Display for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (u, x) in &self.rules {
            write!(f, "{}: ", self.get_name(*u))?;
            x.gfmt(&self.grp, f)?;
            writeln!(f)?;
        }
        write!(f, "")
    }
}

impl fmt::Debug for LQModel {
    fn fmt(&self, ft: &mut fmt::Formatter) -> fmt::Result {
        write!(ft, "{}", self.grp)?;
        for (u, f) in &self.rules {
            let e: &Expr = &f.as_func();

            writeln!(ft, "E{}  : {}", u, e)?;

            let nnf = e.nnf().unwrap_or(e.clone());
            writeln!(ft, "   N: {}", nnf)?;
            let primes = e.prime_implicants();
            writeln!(ft, "   P: {}", primes)?;
        }
        write!(ft, "")
    }
}
