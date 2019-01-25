//! Associate names to their variables

use crate::func::repr::expr::Expr;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

/// Define a group of variables.
///
/// Each variable is identified by a numeric uid. This struct associated
/// human-readable names for the variables.
/// It allows to retrieve and change the name of an existing variable or
/// to retrieve the uid corresponding to a name.
#[derive(Clone)]
pub struct Group {
    name2uid: HashMap<String, usize>,
    uid2name: HashMap<usize, String>,
    cur_uid: usize,
}

lazy_static! {
    static ref RE_UID: Regex = Regex::new(r"[a-zA-Z][a-zA-Z01-9_]*").unwrap();
}

impl Group {
    /// Create a new, empty group
    pub fn new() -> Self {
        Group {
            name2uid: HashMap::new(),
            uid2name: HashMap::new(),
            cur_uid: 0,
        }
    }

    /// Retrieve the uid corresponding to a variable name.
    /// Returns None if the variable name is not defined.
    pub fn node_id(&self, name: &str) -> Option<usize> {
        match self.name2uid.get(name) {
            Some(uid) => Some(*uid),
            None => None,
        }
    }

    /// Retrieve or assign the uid for a variable name.
    /// If the name is not defined, it will associate it to
    /// a new uid.
    /// Returns None if the name is invalid.
    pub fn get_node_id(&mut self, name: &str) -> Option<usize> {
        match self.name2uid.get(name) {
            Some(uid) => return Some(*uid),
            None => (),
        };

        if (!RE_UID.is_match(&name)) {
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

    pub fn get_name(&self, uid: usize) -> String {
        match self.uid2name.get(&uid) {
            Some(name) => name.clone(),
            None => format!("_{}", uid),
        }
    }

    /// Assign a name to an existing variable
    /// Returns false if the name is invalid or already assigned
    /// to another variable
    pub fn set_name(&mut self, uid: usize, name: String) -> bool {
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

    /// Rename a variable.
    /// Returns false if the new name is invalid or already assigned
    /// to another variable
    pub fn rename(&mut self, source: &str, name: String) -> bool {
        // Find the old uid
        match self.name2uid.get(source) {
            None => false,
            Some(u) => self.set_name(*u, name),
        }
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (u, n) in self.uid2name.iter() {
            write!(f, "{}:{}   ", u, n);
        }
        write!(f, "\n")
    }
}
