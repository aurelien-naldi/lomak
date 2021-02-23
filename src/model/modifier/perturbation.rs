use crate::helper::error::{CanFail, GenericError};
use crate::model::QModel;
use crate::variables::GroupedVariables;
use std::collections::{HashMap, HashSet};

pub struct Perturbator<'a> {
    model: &'a mut QModel,
    regulators: HashMap<(usize, usize), bool>,
    components: HashMap<usize, bool>,
    target: Option<usize>,
}

impl<'a> Perturbator<'a> {
    pub fn new(model: &'a mut QModel) -> Self {
        Perturbator {
            model,
            regulators: Default::default(),
            components: Default::default(),
            target: None,
        }
    }

    pub fn guess_lock(&mut self, s: &str, value: bool) -> CanFail<GenericError> {
        if let Some(idx) = s.find("@") {
            self.lock_regulator(&s[..idx], &s[idx + 1..], value)
        } else {
            self.lock_component(&s, value)
        }
    }

    pub fn lock_regulator(&mut self, src: &str, tgt: &str, value: bool) -> CanFail<GenericError> {
        let src = self.model.get_handle_res(src)?;
        let tgt = self.model.get_handle_res(tgt)?;

        // FIXME: handle multivalued

        self.regulators.insert((src, tgt), value);
        Ok(())
    }

    pub fn lock_component(&mut self, uid: &str, value: bool) -> CanFail<GenericError> {
        let uid = self.model.get_handle_res(uid)?;
        self.components.insert(uid, value);
        Ok(())
    }

    pub fn apply(&mut self) {
        // Classical perturbations
        for (uid, value) in &self.components {
            self.model.lock_variable(*uid, *value);
        }

        // Perturbed interactions
        // Collect the modified targets and update their rules
        for t in self.regulators.iter().map(|((_,t),_)| *t).collect::<HashSet<usize>>() {
            self.target = Some(t);
            unimplemented!("TODO: Use the replacer API to rewrite the rules for {}", t);
        }
    }
}
