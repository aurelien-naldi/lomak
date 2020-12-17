//! A logical model is a collection of components associated to Boolean variables and
//! logical rules controlling changes of activity over time, depending on the model state.

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;

use regex::Regex;

use crate::error::EmptyLomakResult;
use crate::func::expr::*;
use crate::func::*;
use crate::model::layout::Layout;
use crate::variables::{check_val, GroupedVariables, ModelVariables, MAXVAL};
use crate::version::{Version, Versionned};

pub mod actions;
pub mod io;
pub mod layout;
pub mod modifier;

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
    rules: Rules,
    layout: Option<Layout>,
    cache: RefCell<ModelCache>,
}

#[derive(Default)]
pub struct Rules {
    rules: HashMap<usize, ComponentRules>,
    version: Version,
}

#[derive(Default)]
struct ModelCache {
    variables: Option<Rc<ModelVariables>>,
    target_rules: Option<Rc<HashMap<usize, Formula>>>,
    local_rules: Option<Rc<HashMap<usize, Formula>>>,
}

#[derive(Clone)]
pub struct FrozenModel {
    variables: Rc<ModelVariables>,
    rules: Rc<HashMap<usize, Formula>>,
}

/// Sharable model reference
#[derive(Clone, Default)]
pub struct SharedModel {
    rc: Rc<RefCell<QModel>>,
}

impl Rules {
    /// Private helper to retrieve and modify the set f rules associated to a component.
    /// This call registers a change since the last version
    fn _ensure(&mut self, cid: usize) -> &mut ComponentRules {
        if !self.rules.contains_key(&cid) {
            self.rules.insert(cid, ComponentRules::new());
        }
        self.version.change();
        self.rules.get_mut(&cid).unwrap()
    }

    /// Replace all rules for the specified component, and return the previous rules if available
    /// This call registers a change since the last version
    fn _replace(&mut self, cid: usize, rule: ComponentRules) -> Option<ComponentRules> {
        self.version.change();
        self.rules.insert(cid, rule)
    }

    /// Retrieve the set of rules for a component if it exists
    fn get(&self, cid: usize) -> Option<&ComponentRules> {
        self.rules.get(&cid)
    }

    /// Assign a Boolean condition for a specific threshold
    fn push(&mut self, cid: usize, value: usize, rule: Formula) {
        self._ensure(cid).push(value, rule);
    }

    /// Restrict the activity of a component
    pub fn restrict_component(&mut self, cid: usize, min: usize, max: usize) {
        self._ensure(cid).restrict(min, max);
    }

    /// Enforce the activity of a specific variable
    pub fn lock_component(&mut self, cid: usize, value: usize) {
        self._ensure(cid).lock(value);
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
        self.rules._ensure(self.variables.component(handle).unwrap());
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
    pub fn frozen(&self) -> FrozenModel {
        FrozenModel {
            variables: self.frozen_variables(),
            rules: self.frozen_rules(),
        }
    }

    pub fn frozen_variables(&self) -> Rc<ModelVariables> {
        self.cache.borrow_mut().get_variables(&self.variables)
    }

    pub fn frozen_rules(&self) -> Rc<HashMap<usize, Formula>> {
        let m = HashMap::new();
        Rc::new(m)
    }

    /// Assign a Boolean condition for a specific threshold
    pub fn push_cpt_rule(&mut self, cid: usize, value: usize, rule: Formula) {
        self.rules.push(cid, value, rule);
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
        let mut expr = self.rules.get(cid).unwrap().raw_variable_formula(value);
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
        self.rules.restrict_component(cid, min, max);
    }

    /// Enforce the activity of a specific variable
    pub fn lock_component(&mut self, cid: usize, value: usize) {
        self.rules.lock_component(cid, value);
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

impl fmt::Display for QModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let namer = self.as_namer();

        for cid in self.components() {
            let name = self.get_name(*cid);
            let rules = self.rules.get(*cid);
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

impl ModelCache {
    fn clear(&mut self) {
        self.variables = None;
        self.clear_rules();
    }

    fn clear_rules(&mut self) {
        self.target_rules = None;
        self.local_rules = None;
    }

    fn get_variables(&mut self, vars: &ModelVariables) -> Rc<ModelVariables> {
        // Update cache if needed
        if self
            .variables
            .as_ref()
            .map(|v| v.version() != vars.version())
            .unwrap_or(true)
        {
            self.variables = Some(Rc::new(vars.clone()));
        }

        Rc::clone(self.variables.as_ref().unwrap())
    }
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
