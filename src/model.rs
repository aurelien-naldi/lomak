//! A logical model is a collection of components associated to Boolean variables and
//! logical rules controlling changes of activity over time, depending on the model state.

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use regex::Regex;

use crate::func::expr::*;
use crate::func::*;
use std::ops::Deref;

pub mod actions;
pub mod io;
pub mod modifier;

/// Maximal number of variables associated to each component
static MAXVAL: usize = 9;

lazy_static! {
    static ref RE_UID: Regex = Regex::new(r"[a-zA-Z][a-zA-Z01-9_]*").unwrap();
}

/// A Boolean variable associated to a qualitative threshold of one of the components
#[derive(Copy, Clone)]
pub struct Variable {
    pub component: usize,
    pub value: usize,
}

/// A formula associated with a target value
#[derive(Clone)]
pub struct Assign {
    pub target: usize,
    pub formula: Formula,
}

/// The list of assignments define the dynamical rules for all variables associated to the same component.
#[derive(Clone)]
pub struct ComponentRules {
    assignments: Vec<Assign>,
}

/// A model contains a list of named components, which are associated to
/// one or several Boolean variables for each qualitative threshold.
/// Components and variables are identified by unique handles (positive integers).
///
/// Finally, each component is associated to a list of Boolean functions defining
/// the conditions required for the activation of each threshold.
#[derive(Default)]
pub struct QModel {
    _next_cpt: usize,
    _next_var: usize,
    components: Vec<usize>,
    variables: Vec<usize>,
    name2uid: HashMap<String, usize>,

    // Connect variables and components
    cpt_variables: HashMap<usize, Vec<usize>>,
    var_component_values: HashMap<usize, Variable>,

    cpt_names: HashMap<usize, String>,
    cpt_rules: HashMap<usize, ComponentRules>,
}

/// Sharable model reference
#[derive(Clone)]
pub struct SharedModel {
    rc: Rc<RefCell<QModel>>,
}

/// Core API for qualitative models
impl QModel {
    /// Find a component by name if it exists.
    ///
    /// Components are associated to a group of related Boolean variables.
    /// The variables can be then used in Boolean expressions, not components.
    pub fn get_component(&self, name: &str) -> Option<usize> {
        if let Some(uid) = self.name2uid.get(name) {
            return Some(*uid);
        }
        None
    }

    /// Find or create a component with a given name
    pub fn ensure_component(&mut self, name: &str) -> usize {
        if let Some(uid) = self.get_component(name) {
            return uid;
        }

        // Create a new component
        let cid = self._next_cpt;
        self._next_cpt += 1;
        self.components.push(cid);
        self.cpt_names.insert(cid, name.to_owned());
        self.cpt_variables.insert(cid, vec![]);
        self.cpt_rules.insert(cid, ComponentRules::new());
        self.name2uid.insert(name.to_owned(), cid);
        cid
    }

    /// Retrieve a variable for an existing component and a threshold value if it exists
    pub fn get_cpt_variable(&self, cid: usize, value: usize) -> Option<usize> {
        let value = check_val(value);
        let variables = self.cpt_variables.get(&cid).unwrap();
        if value > variables.len() {
            return None;
        }
        Some(variables[value - 1])
    }

    /// Find or create a variable for an existing component and a specific threshold value
    pub fn ensure_cpt_variable(&mut self, cid: usize, value: usize) -> usize {
        let value = check_val(value);
        let variables = self.cpt_variables.get_mut(&cid).unwrap();

        // Create new variable(s) as required
        for v in variables.len()..value {
            let vid = self._next_var;
            self._next_var += 1;
            self.variables.push(vid);
            self.var_component_values
                .insert(vid, Variable::new(cid, v + 1));
            variables.push(vid);
        }

        // Return the variable
        variables[value - 1]
    }

    pub fn get_cpt_name(&self, uid: usize) -> String {
        format!("{}", self.cpt_names.get(&uid).unwrap())
    }

    pub fn set_cpt_name(&mut self, uid: usize, name: String) -> bool {
        // Reject invalid new names
        if !RE_UID.is_match(&name) {
            return false;
        }

        // Reject existing names
        if let Some(u) = self.name2uid.get(&name) {
            return *u == uid;
        }

        self.name2uid.remove(self.cpt_names.get(&uid).unwrap());
        self.name2uid.insert(name.clone(), uid);
        self.cpt_names.insert(uid, name);
        true
    }

    pub fn get_var_name(&self, uid: usize) -> String {
        let var = self.var_component_values.get(&uid).unwrap();
        let name = self.cpt_names.get(&var.component).unwrap();
        if var.value != 1 {
            format!("{}:{}", name, var.value)
        } else {
            format!("{}", name)
        }
    }

    pub fn variables(&self) -> &Vec<usize> {
        &self.variables
    }
}

/// Convenience functions, building on the core ones without direct access to the fields
impl QModel {
    /// Rename a component.
    /// Returns false if the new name is invalid or already assigned
    /// to another component
    pub fn rename(&mut self, source: &str, name: String) -> bool {
        match self.get_component(source) {
            None => false,
            Some(u) => self.set_cpt_name(u, name),
        }
    }

    /// Find or create a component with a given name
    pub fn add_component(&mut self, pattern: &str) -> usize {
        if self.get_component(pattern).is_none() {
            return self.ensure_component(pattern);
        };

        let mut inc = 1;
        loop {
            let name = format!("{}_{}", pattern, inc);
            if self.get_component(&name).is_none() {
                return self.ensure_component(&name);
            };
            inc += 1;
        }
    }

    /// Find a variable based on the name of the component and the threshold value
    pub fn find_variable(&self, name: &str, value: usize) -> Option<usize> {
        match self.get_component(name) {
            None => None,
            Some(cid) => self.get_cpt_variable(cid, value),
        }
    }

    /// Find or create a variable with a given component name and threshold value
    pub fn ensure_variable(&mut self, name: &str, value: usize) -> usize {
        let cid = self.ensure_component(name);
        self.ensure_cpt_variable(cid, value)
    }
}

/// Handling of Dynamical rules
impl QModel {
    /// Assign a Boolean condition for a specific threshold
    pub fn push_cpt_rule(&mut self, cid: usize, value: usize, rule: Formula) {
        self.cpt_rules.get_mut(&cid).unwrap().push(value, rule);
    }

    /// Assign a Boolean condition for a specific threshold
    pub fn push_var_rule(&mut self, vid: usize, rule: Formula) {
        let var = self.var_component_values.get(&vid).unwrap();
        let cpt = var.component;
        let val = var.value;
        self.push_cpt_rule(cpt, val, rule);
    }

    pub fn get_var_rule(&self, vid: usize) -> Expr {
        let var = self.var_component_values.get(&vid).unwrap();
        let cid = var.component;
        let value = var.value;
        let mut expr = self
            .cpt_rules
            .get(&cid)
            .unwrap()
            .raw_variable_formula(value);
        let variables = self.cpt_variables.get(&cid).unwrap();

        if value < variables.len() {
            let next_var = variables[value];
            let cur_active = Expr::ATOM(cid);
            let next_active = Expr::ATOM(next_var);
            expr = expr.or(&cur_active.and(&next_active));
        }

        if value > 1 {
            let prev_var = variables[value - 2];
            let prev_active = Expr::ATOM(prev_var);
            expr = expr.and(&prev_active);
        }

        expr.simplify().unwrap_or(expr)
    }
}

impl QModel {
    /// Enforce the activity of a specific variable
    pub fn lock_variable(&mut self, vid: usize, value: bool) {
        let var = &self.var_component_values.get(&vid).unwrap();
        let cpt = var.component;
        let val = var.value;
        if value {
            self.restrict_component(cpt, val, MAXVAL);
        } else {
            self.restrict_component(cpt, 0, val - 1);
        }
    }

    /// Restrict the activity of a component
    pub fn restrict_component(&mut self, cid: usize, min: usize, max: usize) {
        let rules = self.cpt_rules.get_mut(&cid).unwrap();
        rules.restrict(min, max);
    }

    /// Enforce the activity of a specific variable
    pub fn lock_component(&mut self, cid: usize, value: usize) {
        let rules = self.cpt_rules.get_mut(&cid).unwrap();
        rules.lock(value);
    }
}

impl SharedModel {
    pub fn new() -> Self {
        Self {
            rc: Rc::new(RefCell::new(QModel::default())),
        }
    }

    pub fn borrow(&self) -> Ref<QModel> {
        self.rc.as_ref().borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<QModel> {
        self.rc.as_ref().borrow_mut()
    }

    pub fn save(&self, filename: &str, fmt: Option<&str>) -> Result<(), std::io::Error> {
        let model = self.borrow();
        io::save_model(model.deref(), filename, fmt)
    }

    pub fn lock<'a, I: IntoIterator<Item = (&'a str, bool)>>(&self, pairs: I) {
        let mut model = self.borrow_mut();
        for (name, value) in pairs {
            let uid = model.find_variable(&name, 1);
            match uid {
                None => eprintln!("No such variable: {}", name),
                Some(uid) => model.lock_variable(uid, value),
            }
        }
    }
}

impl ComponentRules {
    fn new() -> Self {
        ComponentRules {
            assignments: vec![],
        }
    }

    fn clear(&mut self) {
        self.assignments.clear();
    }

    fn lock(&mut self, value: usize) {
        let value = check_val(value);
        self.clear();
        if value > 0 {
            self.push(value, Formula::from_bool(true));
        }
    }

    fn restrict(&mut self, min: usize, max: usize) {
        let min = check_val(min);
        let max = check_val(max);
        if max <= min {
            self.lock(min);
            return;
        }

        for assign in self.assignments.iter_mut() {
            if assign.target < min {
                assign.target = min;
            } else if assign.target > max {
                assign.target = max;
            }
        }
    }

    fn raw_variable_formula(&self, value: usize) -> Expr {
        let mut expr = Expr::FALSE;
        for asg in self.assignments.iter() {
            let cur: Rc<Expr> = asg.formula.convert_as();
            if asg.target < value {
                expr = expr.and(&cur.not());
            } else {
                expr = expr.or(&cur);
            }
        }
        expr.simplify().unwrap_or(expr)
    }

    pub fn push(&mut self, value: usize, condition: Formula) {
        self.assignments.push(Assign {
            target: value,
            formula: condition,
        })
    }

    pub fn set_formula(&mut self, f: Formula, v: usize) {
        self.clear();
        self.push(v, f);
    }
}

impl Variable {
    fn new(component: usize, value: usize) -> Self {
        Variable { component, value }
    }
}

impl Assign {
    pub fn convert<T: FromBoolRepr>(&self) -> Rc<T> {
        self.formula.convert_as()
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- {}", self.target, self.formula)
    }
}

impl VariableNamer for QModel {
    fn format_name(&self, f: &mut fmt::Formatter, vid: usize) -> fmt::Result {
        let var = self.var_component_values.get(&vid).unwrap();
        let name = self.cpt_names.get(&var.component).unwrap();
        if var.value != 1 {
            write!(f, "{}:{}", name, var.value)
        } else {
            write!(f, "{}", name)
        }
    }

    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }
}

impl fmt::Display for QModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let namer = self.as_namer();

        for cid in self.components.iter() {
            let rules = self.cpt_rules.get(&cid).unwrap();
            let name = self.cpt_names.get(&cid).unwrap();
            for a in rules.assignments.iter() {
                write!(f, "{}", name)?;
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

impl fmt::Debug for QModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for u in self.components.iter() {
            let name = self.cpt_names.get(u).unwrap();
            write!(f, "{} ({}):", u, name)?;
            for v in self.cpt_variables.get(u).unwrap().iter() {
                write!(f, "  {}", v)?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;

        for v in &self.variables {
            let var = self.var_component_values.get(v).unwrap();
            writeln!(f, "{}: {}:{}", v, var.component, var.value)?;
        }
        writeln!(f)?;

        write!(f, "{}", self)
    }
}

fn check_val(value: usize) -> usize {
    if value < 1 {
        eprintln!("Tried to access an impossible value: {}", value);
        return 1;
    }

    if value > MAXVAL {
        eprintln!("Tried to access a large value: {}", value);
        return MAXVAL;
    }
    value
}
