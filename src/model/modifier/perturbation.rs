use crate::func::repr::expr;
use crate::model::modifier::ModelModifier;
use crate::model::LQModel;

pub struct Perturbation {
    model: LQModel,
}

impl Perturbation {
    pub fn new(model: LQModel) -> Self {
        Perturbation { model: model }
    }

    pub fn knockout(mut self, uid: usize) -> Self {
        return self.fix(uid, false);
    }

    pub fn fix(mut self, uid: usize, b: bool) -> Self {
        self.model.set_rule(uid, expr::Expr::from_bool(b));
        return self;
    }
}

impl ModelModifier for Perturbation {
    fn get_model(self) -> LQModel {
        self.model
    }
}
