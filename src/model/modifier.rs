use crate::model::LQModel;

pub mod perturbation;

pub trait ModelModifier {
    fn get_model(self) -> LQModel;
}

struct ModifierDescriptor<T: ModelModifier> {
    name: String,
    builder: Fn(&LQModel) -> T,
}

impl<T: ModelModifier> ModifierDescriptor<T> {
    fn getModifier(&self, _model: &LQModel) {
        //        Box::new(self.builder(model) )
    }
}

pub trait Modifiable {
    fn perturbation(self) -> perturbation::Perturbation;
}

impl Modifiable for LQModel {
    fn perturbation(self) -> perturbation::Perturbation {
        perturbation::Perturbation::new(self)
    }
}
