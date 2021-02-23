use crate::helper::error::{CanFail, GenericError};
use crate::model::QModel;
use crate::variables::GroupedVariables;
use std::collections::HashMap;

pub struct RegulatorLocks<'a> {
    model: &'a mut QModel,
    regulators: HashMap<(usize, usize), bool>,
    components: HashMap<usize, bool>,
    target: Option<usize>,
}

impl<'a> RegulatorLocks<'a> {
    pub fn new(model: &'a mut QModel) -> Self {
        RegulatorLocks {
            model,
            regulators: Default::default(),
            components: Default::default(),
            target: None,
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
        for (uid, value) in &self.components {
            self.model.lock_variable(*uid, *value);
        }

        for ((src, tgt), value) in &self.regulators {
            // TODO: lock regulators
            unimplemented!()
        }
    }
}
