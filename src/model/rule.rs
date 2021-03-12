use crate::func::expr::Expr;
use crate::func::{Formula, FromBoolRepr};
use crate::helper::version::Version;
use crate::variables;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::slice::Iter;

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

#[derive(Default, Clone)]
pub struct Rules {
    rules: HashMap<usize, ComponentRules>,
    cache: RefCell<ModelCache>,
    version: Version,
}

#[derive(Default, Clone)]
struct ModelCache {
    target_rules: Option<Rc<HashMap<usize, Formula>>>,
    local_rules: Option<Rc<HashMap<usize, Formula>>>,
}

impl Rules {
    /// Private helper to retrieve and modify the set of rules associated to a component.
    /// This call registers a change since the last version
    pub fn ensure(&mut self, cid: usize) -> &mut ComponentRules {
        self.rules.entry(cid).or_insert_with(ComponentRules::new);
        self.version.change();
        self.rules.get_mut(&cid).unwrap()
    }

    /// Replace all rules for the specified component, and return the previous rules if available
    /// This call registers a change since the last version
    pub fn replace(&mut self, cid: usize, rule: ComponentRules) -> Option<ComponentRules> {
        self.version.change();
        self.rules.insert(cid, rule)
    }

    /// Retrieve the set of rules for a component if it exists
    pub fn get(&self, cid: usize) -> Option<&ComponentRules> {
        self.rules.get(&cid)
    }

    pub fn get_mut(&mut self, cid: usize) -> Option<&mut ComponentRules> {
        self.rules.get_mut(&cid)
    }

    /// Assign a Boolean condition for a specific threshold
    pub fn push(&mut self, cid: usize, value: usize, rule: Formula) {
        self.ensure(cid).push(value, rule);
    }

    /// Restrict the activity of a component
    pub fn restrict_component(&mut self, cid: usize, min: usize, max: usize) {
        self.ensure(cid).restrict(min, max);
    }

    /// Enforce the activity of a specific variable
    pub fn lock_component(&mut self, cid: usize, value: usize) {
        self.ensure(cid).lock(value);
    }
}

impl ComponentRules {
    fn new() -> Self {
        ComponentRules {
            assignments: vec![],
        }
    }

    pub fn assignments(&self) -> Iter<Assign>{
        self.assignments.iter()
    }

    pub fn map_assignments<F: FnMut(&mut Assign) -> ()>(&mut self, f: F) {
        self.assignments.iter_mut().for_each(f)
    }

    fn clear(&mut self) {
        self.assignments.clear();
    }

    fn lock(&mut self, value: usize) {
        let value = variables::check_tval(value);
        self.clear();
        if value > 0 {
            self.push(value, Formula::from_bool(true));
        }
    }

    fn restrict(&mut self, min: usize, max: usize) {
        let min = variables::check_tval(min);
        let max = variables::check_tval(max);
        if max <= min {
            self.lock(min);
            return;
        }

        if min > 0 {
            // Replace all assignments lower or equal to the new min with a basal rule
            self.assignments.retain(|a| a.target > min);
            self.insert(min, Formula::from_bool(true));
        }
        for assign in self.assignments.iter_mut() {
            if assign.target > max {
                assign.target = max;
            }
        }
    }

    pub fn raw_variable_formula(&self, value: usize) -> Expr {
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

    pub fn insert(&mut self, value: usize, condition: Formula) {
        self.assignments.insert(
            0,
            Assign {
                target: value,
                formula: condition,
            },
        )
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

impl ModelCache {
    fn clear(&mut self) {
        self.target_rules = None;
        self.local_rules = None;
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} <- {}", self.target, self.formula)
    }
}
