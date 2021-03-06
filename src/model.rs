//! Collection of components associated to Boolean variables and dynamical rules.
//! Submodules provide file formats, modifiers as well as analysis tools.
//!
//! Each component has an activity level, represented as the Boolean states of the corresponding
//! Boolean variables. A state of the whole system is then given by the set of all Boolean states.
//! Logical rules define possible changes of activity over time, depending on the current model state.

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;

use crate::func::expr::*;
use crate::func::*;
use crate::helper::error::EmptyLomakResult;
use crate::model::layout::{Layout, NodeLayoutInfo};
use crate::model::rule::Rules;
use crate::variables::{GroupedVariables, ModelVariables, Variable, MAXVAL};

pub mod actions;
pub mod io;
pub mod layout;
pub mod modifier;
pub mod rule;

/// A model contains a list of named components and an associated Boolean variable for each qualitative threshold.
///
/// Finally, each component is associated to a list of Boolean functions defining
/// the conditions required for the activation of each threshold.
#[derive(Default)]
pub struct QModel {
    variables: Rc<ModelVariables>,
    rules: Rc<Rules>,
    layout: Option<Rc<Layout>>,
}

/// Sharable model reference
#[derive(Clone, Default)]
pub struct SharedModel {
    rc: Rc<RefCell<QModel>>,
}

/// Delegate variable handling in models to the dedicated field
impl GroupedVariables for QModel {
    fn get_handle(&self, name: &str) -> Option<usize> {
        self.variables.get_handle(name)
    }

    fn get_component_value(&self, vid: usize) -> Option<&Variable> {
        self.variables.get_component_value(vid)
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
        let handle = Rc::make_mut(&mut self.variables).ensure(name);
        Rc::make_mut(&mut self.rules).ensure(self.variables.component(handle).unwrap());
        handle
    }

    /// Find or create a variable for an existing component and a specific threshold value
    fn ensure_threshold(&mut self, cid: usize, value: usize) -> usize {
        Rc::make_mut(&mut self.variables).ensure_threshold(cid, value)
    }

    fn set_name(&mut self, uid: usize, name: &str) -> Result<bool, &'static str> {
        Rc::make_mut(&mut self.variables).set_name(uid, name)
    }
}

/// Handling of Dynamical rules
impl QModel {
    pub fn frozen_variables(&self) -> Rc<ModelVariables> {
        self.variables.clone()
    }

    pub fn frozen_rules(&self) -> Rc<HashMap<usize, Formula>> {
        let mut m = HashMap::new();
        for u in self.variables() {
            let e = self.get_var_rule(*u);
            m.insert(*u, Formula::from(e));
        }
        Rc::new(m)
    }

    /// Assign a Boolean condition for a specific threshold
    pub fn push_cpt_rule(&mut self, cid: usize, value: usize, rule: Formula) {
        Rc::make_mut(&mut self.rules).push(cid, value, rule);
    }

    pub fn get_layout(&self) -> Option<Rc<Layout>> {
        self.layout.clone()
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
        let mut expr = self
            .rules
            .get(cid)
            .map(|c| c.raw_variable_formula(value))
            .unwrap_or(Expr::FALSE);
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

    pub fn lock_regulator(&mut self, _vid: usize, _target: usize, _value: bool) {
        // FIXME: implement locking of interactions
        unimplemented!()
    }

    /// Restrict the activity of a component
    pub fn restrict_component(&mut self, cid: usize, min: usize, max: usize) {
        Rc::make_mut(&mut self.rules).restrict_component(cid, min, max);
    }

    /// Enforce the activity of a specific variable
    pub fn lock_component(&mut self, cid: usize, value: usize) {
        Rc::make_mut(&mut self.rules).lock_component(cid, value);
    }
}

impl QModel {
    fn layout_mut(&mut self) -> &mut Layout {
        if self.layout.is_none() {
            self.layout = Some(Rc::new(Layout::default()));
        }
        Rc::make_mut(self.layout.as_mut().unwrap())
    }

    pub fn set_bounding_box(&mut self, uid: usize, bb: NodeLayoutInfo) {
        self.layout_mut().set_bounding_box(uid, bb);
    }

    pub fn get_bounding_box(&self, uid: usize) -> Option<&NodeLayoutInfo> {
        match &self.layout {
            None => None,
            Some(l) => l.get_bounding_box(uid),
        }
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
            for a in rules.assignments() {
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
