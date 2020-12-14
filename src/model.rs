//! A logical model is a collection of components associated to Boolean variables and
//! logical rules controlling changes of activity over time, depending on the model state.

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use regex::Regex;

use crate::func::expr::*;
use crate::func::*;
use std::ops::Deref;
use std::slice::Iter;
use crate::error::EmptyLomakResult;

pub mod actions;
pub mod io;
pub mod modifier;

/// Maximal number of variables associated to each component
static MAXVAL: usize = 9;

lazy_static! {
    static ref RE_UID: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z01-9_]*$").unwrap();
    static ref RE_VAR_ID: Regex =
        Regex::new(r"^(?P<cpt>[a-zA-Z][a-zA-Z01-9_]*)(:(?P<th>[1-9]))?$").unwrap();
    static ref EMPTY_USIZE_VEC: Vec<usize> = vec![];
    static ref EMPTY_NAME: String = String::from("");
    static ref DEFAULT_NAME_PATTERN: String = String::from("cpt");
}

/// A Boolean variable associated to a qualitative threshold of one of the components
#[derive(Copy, Clone)]
struct Variable {
    component: usize,
    value: usize,
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
    variables: ModelVariables,
    rules: HashMap<usize, ComponentRules>,

    cached_variables: Cell<Option<Rc<ModelVariables>>>,
}

/// Maintain a list of components and associated variables.
///
/// Each component is associated to one or several variables with ordered thresholds.
/// Both components and variables are identified by unique handles (positive integers)
#[derive(Default)]
pub struct ModelVariables {
    _next_handle: usize,
    _changed: Cell<bool>,

    // Order of components and variables
    components: Vec<usize>,
    variables: Vec<usize>,

    // Find components by name
    name2uid: HashMap<String, usize>,

    // Connect variables and components
    cpt_to_variables: HashMap<usize, Vec<usize>>,
    var_to_cpt_value: HashMap<usize, Variable>,

    names: HashMap<usize, String>,
}

/// Sharable model reference
#[derive(Clone,Default)]
pub struct SharedModel {
    rc: Rc<RefCell<QModel>>,
}

pub enum GroupedVariableError {
    UnknownComponent,
    InvalidName,
    NameAlreadyExists,
}

pub trait GroupedVariables {
    /// Find a variable by name if it exists.
    fn get_handle(&self, name: &str) -> Option<usize>;

    /// Retrieve the name of a variable
    /// If the handle corresponds to a variable associated to threshold 1, this corresponds to the name of the component,
    /// otherwise, the threshold is indicated as suffix
    /// Invalid handles yield an empty name
    fn get_name(&self, handle: usize) -> &str;

    /// Retrieve the list of variables associated to a given component
    /// Invalid handles yield an empty list
    fn get_variables(&self, handle: usize) -> &Vec<usize>;

    /// Retrieve a variable for an existing component and a threshold value if it exists
    fn get_variable(&self, handle: usize, value: usize) -> Option<usize> {
        let variables = self.get_variables(handle);
        if value > 0 && value < variables.len() {
            Some(variables[value - 1])
        } else {
            None
        }
    }

    fn components(&self) -> Iter<usize>;

    fn variables(&self) -> Iter<usize>;

    /// Find or create a component with a given name.
    ///
    /// If the name is invalid, a normalized name will be used, exact behaviour
    /// is still undefined:
    /// * normalize before looking for a match may match a different existing component,
    /// * normalize and use as pattern: may create multiple components for the same original name,
    /// * keep track of previous invalid names: can add some noise and corner cases.
    fn ensure(&mut self, name: &str) -> usize;

    /// Find or create a variable for an existing component and a specific threshold value.
    ///
    /// Invalid handles trigger the creation of a new component.
    fn ensure_threshold(&mut self, handle: usize, value: usize) -> usize;

    /// Change the name of a component.
    ///
    /// The selected name will be used for the first variable, all associated variable
    /// take the corresponding extended name (with the threshold suffix).
    ///
    /// Invalid handles trigger the creation of a new component
    fn set_name(&mut self, handle: usize, name: &str) -> Result<bool, &'static str>;

    /// Rename a component.
    /// Returns false if the new name is invalid or already assigned
    /// to a different component
    fn rename(&mut self, source: &str, name: &str) -> Result<bool, &'static str> {
        match self.get_handle(source) {
            None => Err("Unknown component"),
            Some(u) => self.set_name(u, name),
        }
    }

    /// Find or create a component with a given naming pattern
    fn add_component(&mut self, pattern: &str) -> usize {
        match self.find_free_name(pattern) {
            None => self.ensure(pattern),
            Some(n) => self.ensure(&n),
        }
    }

    fn find_free_name(&self, pattern: &str) -> Option<String> {
        if self.get_handle(pattern).is_none() {
            return None;
        };
        let mut inc = 1;
        loop {
            let name = format!("{}_{}", pattern, inc);
            if self.get_handle(&name).is_none() {
                return Some(name);
            };
            inc += 1;
        }
    }
}

impl ModelVariables {
    fn set_change(&self, b: bool) {
        self._changed.set(b);
    }
    fn has_changed(&self) -> bool {
        self._changed.get()
    }

    fn variable(&self, h: usize) -> Option<&Variable> {
        self.var_to_cpt_value.get(&h)
    }

    fn component(&self, h: usize) -> Option<usize> {
        self.variable(h).map(|v| v.component)
    }

    /// Make sure that a handle exists
    fn ensure_handle(&mut self, handle: usize) {
        if let Some(var) = self.var_to_cpt_value.get(&handle) {
            return;
        }

        eprintln!(
            "Warning: enforced creation of a component for handle {}",
            handle
        );
        let name = self.find_free_name("v");
        self._create_component(handle, name.as_ref().unwrap_or(&DEFAULT_NAME_PATTERN));
    }

    /// Create a new component.
    ///
    /// This internal function should only be called
    fn _create_component(&mut self, handle: usize, name: &str) {
        if self.names.contains_key(&handle) {
            panic!("The component already exists");
        }

        if handle >= self._next_handle {
            self._next_handle = handle + 1;
        }

        self.set_change(true);
        self.components.push(handle);
        self.names.insert(handle, name.to_owned());
        self.cpt_to_variables.insert(handle, vec![handle]);
        self.name2uid.insert(name.to_owned(), handle);
        self.var_to_cpt_value.insert(
            handle,
            Variable {
                component: handle,
                value: 1,
            },
        );
    }
}

impl GroupedVariables for ModelVariables {
    fn get_handle(&self, name: &str) -> Option<usize> {
        if let Some(h) = self.name2uid.get(name) {
            return Some(*h);
        }
        None
    }

    fn get_name(&self, handle: usize) -> &str {
        &self.names.get(&handle).unwrap_or(&EMPTY_NAME)
    }

    fn get_variables(&self, handle: usize) -> &Vec<usize> {
        let cpt = self.component(handle).unwrap_or(handle);
        self.cpt_to_variables.get(&cpt).unwrap_or(&EMPTY_USIZE_VEC)
    }

    fn components(&self) -> Iter<usize> {
        self.components.iter()
    }

    fn variables(&self) -> Iter<usize> {
        self.variables.iter()
    }

    fn ensure(&mut self, name: &str) -> usize {
        if let Some(uid) = self.get_handle(name) {
            return uid;
        }

        let cap = match RE_VAR_ID.captures(&name) {
            None => panic!("Invalid name: {}", name),
            Some(c) => c,
        };

        // Retrieve or create the component
        let cpt_name = cap.name("cpt").unwrap().as_str();
        let cid = match self.get_handle(cpt_name) {
            Some(c) => c,
            None => {
                // Create a new component
                let cid = self._next_handle;
                self._create_component(self._next_handle, cpt_name);
                cid
            }
        };

        match cap.name("th") {
            None => cid,
            Some(t) => self.ensure_threshold(cid, t.as_str().parse().unwrap()),
        }
    }

    fn ensure_threshold(&mut self, vid: usize, value: usize) -> usize {
        let value = check_val(value);
        self.ensure_handle(vid);
        let cid = self.component(vid).unwrap();
        let variables = self.cpt_to_variables.get_mut(&cid).unwrap();
        let cptname = self.names.get(&cid).unwrap().to_string();

        // Create new variable(s) as required
        let mut changed = false;
        for v in variables.len()..value {
            let vid = self._next_handle;
            self._next_handle += 1;
            changed = true;
            self.names.insert(vid, format!("{}:{}", cptname, v + 1));
            self.var_to_cpt_value.insert(vid, Variable::new(cid, v + 1));
            variables.push(vid);
        }
        if changed {
            self.set_change(true);
        }
        // Return the variable
        self.cpt_to_variables.get(&cid).unwrap()[value - 1]
    }

    fn set_name(&mut self, h: usize, name: &str) -> Result<bool, &'static str> {
        // Reject invalid new names
        if !RE_UID.is_match(&name) {
            return Err("Invalid name");
        }

        self.ensure_handle(h);
        let ch = self.component(h).unwrap();

        // Reject existing names
        if let Some(u) = self.name2uid.get(name) {
            return if *u == ch {
                Ok(true)
            } else {
                Err("Name already exists")
            };
        }

        // Update the names of all associated variables
        let variables = self.cpt_to_variables.get(&ch).unwrap();
        for (i, v) in variables.iter().enumerate() {
            self.name2uid.remove(self.names.get(v).unwrap());
            let newname = if *v == 0 {
                String::from(name)
            } else {
                format!("{}:{}", name, i)
            };
            self.name2uid.insert(newname.clone(), *v);
            self.names.insert(*v, newname);
        }

        Ok(true)
    }
}

/// Delegate variable handling in models to the dedicated field
impl GroupedVariables for QModel {
    fn get_handle(&self, name: &str) -> Option<usize> {
        self.variables.get_handle(name)
    }

    fn get_name(&self, uid: usize) -> &str {
        self.variables.get_name(uid)
    }

    fn get_variables(&self, cid: usize) -> &Vec<usize> {
        self.variables.get_variables(cid)
    }

    fn components(&self) -> Iter<usize> {
        self.variables.components()
    }

    fn variables(&self) -> Iter<usize> {
        self.variables.variables()
    }

    fn ensure(&mut self, name: &str) -> usize {
        let handle = self.variables.ensure(name);
        let cid = self.variables.component(handle).unwrap();
        if !self.rules.contains_key(&cid) {
            self.rules.insert(cid, ComponentRules::new());
        }
        handle
    }

    /// Find or create a variable for an existing component and a specific threshold value
    fn ensure_threshold(&mut self, cid: usize, value: usize) -> usize {
        self.variables.ensure_threshold(cid, value)
    }

    fn set_name(&mut self, uid: usize, name: &str) -> Result<bool, &'static str> {
        self.variables.set_name(uid, name)
    }
}

/// Handling of Dynamical rules
impl QModel {
    fn get_rules(&self, handle: usize) {
        let cid = self.variables.component(handle).unwrap_or(handle);
        let rules = self.rules.get(&cid);
    }

    /// Assign a Boolean condition for a specific threshold
    pub fn push_cpt_rule(&mut self, cid: usize, value: usize, rule: Formula) {
        self.rules.get_mut(&cid).unwrap().push(value, rule);
    }

    /// Assign a Boolean condition for a specific threshold
    pub fn push_var_rule(&mut self, vid: usize, rule: Formula) {
        let var = self.variables.variable(vid);
        if let Some(var) = var {
            let cpt = var.component;
            let val = var.value;
            self.push_cpt_rule(cpt, val, rule);
        }
    }

    pub fn get_var_rule(&self, vid: usize) -> Expr {
        let var = self.variables.variable(vid);
        if var.is_none() {
            eprintln!("Variable not found {}", vid);
            return Expr::FALSE;
        }
        let var = var.unwrap();
        let cid = var.component;
        let value = var.value;
        let mut expr = self.rules.get(&cid).unwrap().raw_variable_formula(value);
        let variables = self.get_variables(cid);

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
        let var = &self.variables.variable(vid);
        if var.is_none() {
            return;
        }
        let var = var.unwrap();
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
        let rules = self.rules.get_mut(&cid).unwrap();
        rules.restrict(min, max);
    }

    /// Enforce the activity of a specific variable
    pub fn lock_component(&mut self, cid: usize, value: usize) {
        let rules = self.rules.get_mut(&cid).unwrap();
        rules.lock(value);
    }
}

impl SharedModel {
    pub fn with(model: QModel) -> Self {
        Self {
            rc: Rc::new(RefCell::new(model)),
        }
    }

    pub fn borrow(&self) -> Ref<QModel> {
        self.rc.as_ref().borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<QModel> {
        self.rc.as_ref().borrow_mut()
    }

    pub fn save(&self, filename: &str, fmt: Option<&str>) -> EmptyLomakResult {
        let model = self.borrow();
        io::save_model(model.deref(), filename, fmt)
    }

    pub fn lock<'a, I: IntoIterator<Item = (&'a str, bool)>>(&self, pairs: I) {
        let mut model = self.borrow_mut();
        for (name, value) in pairs {
            let uid = model.get_handle(&name);
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

impl<T> VariableNamer for T
where
    T: GroupedVariables,
{
    fn format_name(&self, f: &mut fmt::Formatter, vid: usize) -> fmt::Result {
        write!(f, "{}", self.get_name(vid))
    }

    fn as_namer(&self) -> &dyn VariableNamer {
        self
    }
}

impl fmt::Display for QModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let namer = self.as_namer();

        for cid in self.components() {
            let name = self.get_name(*cid);
            let rules = self.rules.get(&cid);
            if rules.is_none() {
                write!(f, "{}: FALSE", name)?;
                continue;
            }
            let rules = rules.unwrap();
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
        for u in self.components() {
            let name = self.get_name(*u);
            write!(f, "{} ({}):", u, name)?;
            for v in self.get_variables(*u).iter() {
                write!(f, "  {}", v)?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;

        for v in self.variables() {
            let var = self.variables.variable(*v).unwrap();
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::model::actions::fixpoints::*;
    use crate::model::actions::trapspaces::*;

    #[test]
    fn the_regex() {
        let smodel = SharedModel::default();
        {
            let mut model = smodel.borrow_mut();

            let v1 = model.ensure("var1");
            let vt = model.ensure("test");
            let vg = model.ensure("GATA3");
            let vt2 = model.ensure("test:2");
            let vf = model.ensure("Foxp3");
            let vf = model.ensure_threshold(17, 2);
            let v1 = model.ensure("pipo");

            println!("{:#?}", model.get_handle("v"));

            println!("{:#?}", model);
        }

        let fp = TrapspacesBuilder::new(smodel).solve(None);

        println!("trap spaces: {}", fp);
    }
}
