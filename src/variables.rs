use std::collections::HashMap;
use std::fmt;

use regex::Regex;

use crate::func::*;
use crate::helper::version::{Version, Versionned};
use std::slice::Iter;

/// Maximal number of variables associated to each component
pub static MAXVAL: usize = 9;

lazy_static! {
    static ref RE_UID: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z01-9_]*$").unwrap();
    static ref RE_VAR_ID: Regex =
        Regex::new(r"^(?P<cpt>[a-zA-Z][a-zA-Z01-9_]*)(:(?P<th>[1-9]))?$").unwrap();
    static ref EMPTY_USIZE_VEC: Vec<usize> = vec![];
    static ref EMPTY_NAME: String = String::from("");
    static ref DEFAULT_NAME_PATTERN: String = String::from("cpt");
}

/// Maintain a list of components and associated variables.
///
/// Each component is associated to one or several variables with ordered thresholds.
/// Both components and variables are identified by unique handles (positive integers)
#[derive(Default, Clone)]
pub struct ModelVariables {
    _next_handle: usize,
    _version: Version,

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

/// A Boolean variable associated to a qualitative threshold of one of the components
#[derive(Copy, Clone)]
pub struct Variable {
    pub component: usize,
    pub value: usize,
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
    pub fn variable(&self, h: usize) -> Option<&Variable> {
        self.var_to_cpt_value.get(&h)
    }

    pub fn component(&self, h: usize) -> Option<usize> {
        self.variable(h).map(|v| v.component)
    }

    /// Make sure that a handle exists
    fn ensure_handle(&mut self, handle: usize) {
        if self.var_to_cpt_value.get(&handle).is_some() {
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

        self.changed();
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

    fn changed(&mut self) {
        self._version.change();
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
            self.changed();
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

impl Versionned for ModelVariables {
    fn version(&self) -> usize {
        self._version.version()
    }
}

impl Variable {
    pub fn new(component: usize, value: usize) -> Self {
        Variable { component, value }
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

pub fn check_val(value: usize) -> usize {
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
