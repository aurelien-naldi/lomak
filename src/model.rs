//! Logical model: collections of functions

use std::collections::HashMap;
use regex::Regex;
use std::fmt;

use crate::func;
use crate::func::expr::Expr;
use crate::func::variables;
use crate::func::variables::VariableNamer;
use crate::func::Grouped;

pub mod actions;
pub mod io;
pub mod modifier;

lazy_static! {
    static ref RE_PRT: Regex = Regex::new(r"([a-zA-Z][a-zA-Z01-9_]*)%([01])").unwrap();
}

pub struct LQModel {
    grp: variables::Group,
    rules: HashMap<usize, func::Formula>,
}

impl LQModel {
    pub fn new() -> LQModel {
        LQModel {
            grp: variables::Group::new(),
            rules: HashMap::new(),
        }
    }

    pub fn set_rule(&mut self, target: usize, rule: Expr) {
        if let Some(f) = self.rules.get_mut(&target) {
            f.set_expr(rule);
            return;
        }
        self.rules.insert(target, func::Formula::from_expr(rule));
    }

    pub fn knockout(mut self, uid: usize) -> Self {
        self.set_rule(uid, Expr::FALSE);
        self
    }

    pub fn knockin(mut self, uid: usize) -> Self {
        self.set_rule(uid, Expr::TRUE);
        self
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
                        },
                    }
                }
            }
        }
        self
    }

    #[allow(dead_code)]
    pub fn extend_rule(&mut self, target: usize, rule: Expr) {
        match self.rules.remove(&target) {
            None => self.set_rule(target, rule),
            Some(r) => self.set_rule(target, r.as_expr().or(&rule)),
        }
    }

    pub fn rules(&self) -> &HashMap<usize, func::Formula> {
        &self.rules
    }
}

/// Delegate the VariableNamer trait to the internal Group
impl VariableNamer for LQModel {
    fn node_id(&self, name: &str) -> Option<usize> {
        self.grp.node_id(name)
    }

    fn get_node_id(&mut self, name: &str) -> Option<usize> {
        self.grp.get_node_id(name)
    }

    fn get_name(&self, uid: usize) -> String {
        self.grp.get_name(uid)
    }

    fn set_name(&mut self, uid: usize, name: String) -> bool {
        self.grp.set_name(uid, name)
    }

    fn rename(&mut self, source: &str, name: String) -> bool {
        self.grp.rename(source, name)
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
            let e: &Expr = &f.as_expr();

            writeln!(ft, "E{}  : {}", u, e)?;

            let nnf = e.nnf().unwrap_or(e.clone());
            writeln!(ft, "   N: {}", nnf)?;
            let primes = e.prime_implicants();
            writeln!(ft, "   P: {}", primes)?;
        }
        write!(ft, "")
    }
}
