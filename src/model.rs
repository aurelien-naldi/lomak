//! A logical model is a collection of components associated to Boolean variables and
//! logical rules controlling changes of activity over time, depending on the model state.

use std::cell::{Ref, RefCell, RefMut, Cell};
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
struct Assign {
    pub target: usize,
    pub formula: Formula,
}

/// The list of assignments define the dynamical rules for all variables associated to the same component.
#[derive(Clone)]
struct ComponentRules {
    assignments: Vec<Assign>,
}

/// A model contains a list of named components and an associated Boolean variable for each qualitative threshold.
///
/// Finally, each component is associated to a list of Boolean functions defining
/// the conditions required for the activation of each threshold.
#[derive(Default)]
pub struct QModel {
    cpt_variables: ModelVariables,
    cpt_rules: HashMap<usize, ComponentRules>,

    cached_variables: Cell<Option<Rc<ModelVariables>>>,
}

/// Maintain a list of components and associated variables.
///
/// Each component is associated to one or several variables with ordered thresholds.
/// Both components and variables are identified by unique handles (positive integers)
#[derive(Default)]
pub struct ModelVariables {
    _next_cpt: usize,
    _next_var: usize,
    components: Vec<usize>,
    variables: Vec<usize>,
    name2uid: HashMap<String, usize>,

    // Connect variables and components
    cpt_variables: HashMap<usize, Vec<usize>>,
    var_component_values: HashMap<usize, Variable>,

    cpt_names: HashMap<usize, String>,
    var_names: HashMap<usize, String>,
}

/// Sharable model reference
#[derive(Clone)]
pub struct SharedModel {
    rc: Rc<RefCell<QModel>>,
}

pub trait GroupedVariables {
    /// Find a component by name if it exists.
    ///
    /// Components are associated to a group of related Boolean variables.
    /// The variables can be then used in Boolean expressions, not components.
    fn get_component(&self, name: &str) -> Option<usize>;

    /// Find or create a component with a given name
    fn ensure_component(&mut self, name: &str) -> usize;

    /// Retrieve a variable for an existing component and a threshold value if it exists
    fn get_cpt_variable(&self, cid: usize, value: usize) -> Option<usize>;

    /// Find or create a variable for an existing component and a specific threshold value
    fn ensure_cpt_variable(&mut self, cid: usize, value: usize) -> usize;

    fn get_cpt_name(&self, uid: usize) -> &str;

    /// Retrieve the list of variables associated to a given component
    fn get_cpt_variables(&self, cid: usize) -> &Vec<usize>;

    fn get_var_name(&self, vid: usize) -> &str;

    fn variable(&self, vid: usize) -> &Variable;

    fn variables(&self) -> &Vec<usize>;

    fn components(&self) -> &Vec<usize>;

    /// Rename a component.
    /// Returns false if the new name is invalid or already assigned
    /// to another component
    fn rename(&mut self, source: &str, name: String) -> bool {
        match self.get_component(source) {
            None => false,
            Some(u) => self.set_cpt_name(u, name),
        }
    }

    /// Find or create a component with a given name
    fn add_component(&mut self, pattern: &str) -> usize {
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
    fn find_variable(&self, name: &str, value: usize) -> Option<usize> {
        match self.get_component(name) {
            None => None,
            Some(cid) => self.get_cpt_variable(cid, value),
        }
    }

    /// Find or create a variable with a given component name and threshold value
    fn ensure_variable(&mut self, name: &str, value: usize) -> usize {
        let cid = self.ensure_component(name);
        self.ensure_cpt_variable(cid, value)
    }

    fn set_cpt_name(&mut self, uid: usize, name: String) -> bool;

}

impl GroupedVariables for ModelVariables {
    fn get_component(&self, name: &str) -> Option<usize> {
        if let Some(uid) = self.name2uid.get(name) {
            return Some(*uid);
        }
        None
    }

    fn ensure_component(&mut self, name: &str) -> usize {
        if let Some(uid) = self.get_component(name) {
            return uid;
        }

        // Create a new component
        let cid = self._next_cpt;
        self._next_cpt += 1;
        self.components.push(cid);
        self.cpt_names.insert(cid, name.to_owned());
        self.cpt_variables.insert(cid, vec![]);
        self.name2uid.insert(name.to_owned(), cid);
        cid
    }

    fn get_cpt_variable(&self, cid: usize, value: usize) -> Option<usize> {
        let value = check_val(value);
        let variables = self.cpt_variables.get(&cid).unwrap();
        if value > variables.len() {
            return None;
        }
        Some(variables[value - 1])
    }

    fn ensure_cpt_variable(&mut self, cid: usize, value: usize) -> usize {
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

    fn get_cpt_name(&self, uid: usize) -> &str {
        &self.cpt_names.get(&uid).unwrap()
    }

    fn set_cpt_name(&mut self, uid: usize, name: String) -> bool {
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

    fn get_var_name(&self, vid: usize) -> &str {
        if let Some(name) = self.var_names.get(&vid) {
            return name;
        }
        if let Some(var) = self.var_component_values.get(&vid) {
            if var.value != 1 {
                panic!("A better name should have been cached for {} = [{}:{}]", vid, var.component, var.value);
            }
            return self.get_cpt_name(var.component);
        }
        panic!("Unknown variable {}", vid);
    }

    fn get_cpt_variables(&self, cid: usize) -> &Vec<usize> {
        self.cpt_variables.get(&cid).unwrap()
    }

    fn variable(&self, vid: usize) -> &Variable {
        self.var_component_values.get(&vid).unwrap()
    }

    fn variables(&self) -> &Vec<usize> {
        &self.variables
    }

    fn components(&self) -> &Vec<usize> {
        &self.components
    }
}

/// Delegate variable handling in models to the dedicated field
impl GroupedVariables for QModel {
    fn get_component(&self, name: &str) -> Option<usize> {
        self.cpt_variables.get_component(name)
    }

    fn ensure_component(&mut self, name: &str) -> usize {
        if let Some(uid) = self.get_component(name) {
            return uid;
        }

        // Create a new component
        let cid = self.cpt_variables.ensure_component(name);
        self.cpt_rules.insert(cid, ComponentRules::new());
        cid
    }

    /// Retrieve a variable for an existing component and a threshold value if it exists
    fn get_cpt_variable(&self, cid: usize, value: usize) -> Option<usize> {
        self.cpt_variables.get_cpt_variable(cid, value)
    }

    /// Find or create a variable for an existing component and a specific threshold value
    fn ensure_cpt_variable(&mut self, cid: usize, value: usize) -> usize {
        self.cpt_variables.ensure_cpt_variable(cid, value)
    }

    fn get_cpt_name(&self, uid: usize) -> &str {
        self.cpt_variables.get_cpt_name(uid)
    }

    fn set_cpt_name(&mut self, uid: usize, name: String) -> bool {
        self.cpt_variables.set_cpt_name(uid, name)
    }

    fn get_var_name(&self, vid: usize) -> &str {
        self.cpt_variables.get_var_name(vid)
    }

    fn get_cpt_variables(&self, cid: usize) -> &Vec<usize> {
        self.cpt_variables.get_cpt_variables(cid)
    }

    fn variable(&self, vid: usize) -> &Variable {
        self.cpt_variables.variable(vid)
    }

    fn variables(&self) -> &Vec<usize> {
        self.cpt_variables.variables()
    }

    fn components(&self) -> &Vec<usize> {
        self.cpt_variables.components()
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
        let var = self.variable(vid);
        let cpt = var.component;
        let val = var.value;
        self.push_cpt_rule(cpt, val, rule);
    }

    pub fn get_var_rule(&self, vid: usize) -> Expr {
        let var = self.variable(vid);
        let cid = var.component;
        let value = var.value;
        let mut expr = self
            .cpt_rules
            .get(&cid)
            .unwrap()
            .raw_variable_formula(value);
        let variables = self.get_cpt_variables(cid);

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
        let var = &self.variable(vid);
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

impl<T> VariableNamer for T where T: GroupedVariables {
    fn format_name(&self, f: &mut fmt::Formatter, vid: usize) -> fmt::Result {
        write!(f, "{}", self.get_var_name(vid))
    }

    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }
}

impl fmt::Display for QModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let namer = self.as_namer();

        for cid in self.components().iter() {
            let rules = self.cpt_rules.get(&cid).unwrap();
            let name = self.get_cpt_name(*cid);
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
        for u in self.components().iter() {
            let name = self.get_cpt_name(*u);
            write!(f, "{} ({}):", u, name)?;
            for v in self.get_cpt_variables(*u).iter() {
                write!(f, "  {}", v)?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;

        for v in self.variables() {
            let var = self.variable(*v);
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
