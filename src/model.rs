//! Logical model: collections of functions

use std::collections::HashMap;

use std::fmt;

use crate::func;
use crate::func::expr::Expr;
use crate::func::paths::PathsMerger;
use crate::func::variables;
use crate::func::Grouped;

pub mod actions;
pub mod io;
pub mod modifier;

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

    #[allow(dead_code)]
    pub fn extend_rule(&mut self, target: usize, rule: Expr) {
        match self.rules.remove(&target) {
            None => self.set_rule(target, rule),
            Some(r) => self.set_rule(target, r.as_expr().or(&rule)),
        }
    }

    pub fn nnf(&self) {
        for (u, f) in &self.rules {
            let e = f.as_expr();
            println!("{}: {}", u, e.nnf().unwrap_or(Expr::clone(&e)));
        }
    }

    pub fn primes(&self) {
        for (u, f) in &self.rules {
            let primes = f.as_expr().prime_implicants();
            println!("PI {}: {}", u, primes);
        }
    }

    pub fn json_primes(&self) {
        println!("{{");
        let mut first = true;
        for (u, f) in &self.rules {
            if first {
                first = false;
            } else {
                println!(",");
            }
            let name = self.grp.get_name(*u);
            let pos_primes = f.as_expr().prime_implicants();
            let neg_primes = f.as_expr().not().prime_implicants();
            println!("\"{}\":[", name);
            neg_primes.to_json(&self.grp);
            println!(",");
            pos_primes.to_json(&self.grp);
            print!("]");
        }
        println!("\n}}");
    }

    pub fn stable(&self) {
        for (u, f) in &self.rules {
            let cur = Expr::ATOM(*u);
            let e = &f.as_expr();
            let condition = cur.and(e).or(&cur.not().and(&e.not()));
            let primes = condition.prime_implicants();
            println!("S PI {}: {}", u, primes);
        }
    }

    pub fn stable_full(&self, _go: bool) {
        let mut merger = PathsMerger::new();
        for (u, f) in &self.rules {
            let cur = Expr::ATOM(*u);
            let e = &f.as_expr();
            let condition = cur.and(e).or(&cur.not().and(&e.not()));
            if !merger.add(&condition.prime_implicants()) {
                println!("No solution!");
                return;
            }
        }

        println!("Needs further merging...");
        //        primes.sort_by(|p1,p2| p1.len().cmp(&p2.len()));
    }

    pub fn dbg(&self) {
        println!("{}", self.grp);
        for (u, f) in &self.rules {
            let e: &Expr = &f.as_expr();

            println!("E{}  : {}", u, e);

            let nnf = e.nnf().unwrap_or(e.clone());
            println!("   N: {}", nnf);

            //            let fd = e.dissolve(true).unwrap_or(e.clone());
            //            println!("   D: {}", fd);

            //            let fdn = nnf.dissolve(true).unwrap_or(nnf.clone());
            //            println!("   D: {}", fdn);

            let primes = e.prime_implicants();
            println!("   P: {}", primes);

            println!();
        }
    }

    pub fn get_node_id(&mut self, name: &str) -> Option<usize> {
        self.grp.get_node_id(name)
    }

    #[allow(dead_code)]
    pub fn rename(&mut self, source: &str, name: String) -> bool {
        self.grp.rename(source, name)
    }
}

impl fmt::Display for LQModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (u, x) in &self.rules {
            write!(f, "{}: ", u)?;
            x.gfmt(&self.grp, f)?;
            writeln!(f)?;
        }
        write!(f, "")
    }
}
