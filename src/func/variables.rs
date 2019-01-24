use crate::func::repr::expr::Expr;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub struct Var {
    pub uid: usize,
}

#[derive(Clone)]
pub struct Group {
    name2uid: HashMap<String, usize>,
    uid2name: HashMap<usize, String>,
    cur_uid: usize,
}

impl Group {
    pub fn new() -> Self {
        Group {
            name2uid: HashMap::new(),
            uid2name: HashMap::new(),
            cur_uid: 0,
        }
    }

    pub fn get_node_id(&mut self, name: &str) -> usize {
        match self.name2uid.get(name) {
            Some(uid) => return *uid,
            None => (),
        }
        let name = String::from(name);
        let uid = self.cur_uid;
        self.cur_uid += 1;
        let ret = uid;
        self.name2uid.insert(name.clone(), uid);
        self.uid2name.insert(uid, name);
        ret
    }

    pub fn get_var(&self, uid: usize) -> Var {
        Var { uid: uid }
    }

    pub fn get_var_from_name(&mut self, name: &str) -> Var {
        let uid = self.get_node_id(name);
        self.get_var(uid)
    }

    pub fn get_name(&self, uid: usize) -> String {
        match self.uid2name.get(&uid) {
            Some(name) => return name.clone(),
            None => format!("_{}", uid),
        }
    }

    pub fn rename(&mut self, source: &str, name: String) -> bool {
        let uid = match self.name2uid.get(source) {
            None => return false,
            Some(v) => *v,
        };

        self.name2uid.remove(source);
        self.name2uid.insert(name.clone(), uid);
        self.uid2name.insert(uid, name);
        true
    }
}

impl Var {
    pub fn as_expr(self) -> Expr {
        Expr::ATOM(self)
    }

    fn fmt_in_group(&self, f: &mut fmt::Formatter, grp: Group) -> fmt::Result {
        write!(f, "{}", grp.get_name(self.uid))
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}", self.uid)
    }
}

impl PartialEq for Var {
    fn eq(&self, other: &Var) -> bool {
        self.uid == other.uid
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
