#![feature(specialization, const_fn)]

use pyo3::{PyResult, exceptions};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::class::basic::PyObjectProtocol;

use lomak::model;
use lomak::model::actions::*;
use lomak::model::io;


#[pyfunction]
/// Create a new instance
fn new_model() -> LModel {
    LModel{ m: model::new_model() }
}

#[pyfunction]
/// Create a new instance
fn load_model(filename: &str) -> PyResult<LModel> {
    match io::load_model(filename, None) {
        Ok(m) => Ok(LModel{m: m}),
        Err(e) => Err(exceptions::ValueError::py_err(format!("{}",e))),
    }
}

/// Simple python wrapper for our rust code
#[pymodule]
fn lomak(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(new_model))?;
    m.add_wrapped(wrap_pyfunction!(load_model))?;
    Ok(())
}

/// Wrap a Logical model as a Python object
#[pyclass]
struct LModel {
    m: model::LQModelRef
}

/// Wrap a Logical model as a Python object
#[pyclass]
struct ModelAction {
    m: Box<dyn model::actions::ActionBuilder>
}


#[pymethods]
impl LModel {

    #[new]
    fn new(obj: &PyRawObject) {
        obj.init({
            LModel { m: model::new_model() }
        });
    }

    /// Save a model to file
    fn save(&self, filename: &str) {
        io::save_model(self.m.as_ref(), filename, None);
    }

    /// Lock a component of the model
    fn lock(&mut self, name: &str, value: bool) -> bool {
        let model = self.m.as_mut();
        if let Some(uid) = model.get_component(name) {
            model.lock(uid, value);
            return true
        }
        false
    }

    /// Rename a component of the model
    fn rename(&mut self, old_name: &str, new_name: &str) -> bool {
        let model = self.m.as_mut();
        model.rename(old_name, new_name.to_owned())
    }

    fn primes(&self) {
        let builder = model::actions::primes::PrimeBuilder::new(self.m.as_ref());
        builder.call();
    }

    fn fixpoints(&self) {
        let builder = model::actions::stable::FixedBuilder::new(self.m.as_ref());
        builder.call();
    }

    fn trapspaces(&self) {
        let builder = model::actions::trapspaces::TrapspacesBuilder::new(self.m.as_ref());
        builder.call();
    }

}

#[pyproto]
impl PyObjectProtocol<'_> for LModel {


    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.m.for_display()))
    }
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

}
