//#![feature(specialization)]

use pyo3::{PyResult, exceptions};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::class::basic::PyObjectProtocol;

use lomak::func;
use lomak::model;
use lomak::model::actions::*;
use lomak::model::io;

/// Simple python wrapper for our rust code
#[pymodule]
fn lomak(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Model>()?;
    Ok(())
}

/// Wrap a Logical model as a Python object
#[pyclass(module="lomak")]
pub struct Model {
    m: model::LQModelRef
}

/// Python API for Boolean expressions
#[pyclass(module="lomak")]
pub struct Expr {
    expr: func::expr::Expr,
}

/// Wrap any model action as a Python object
#[pyclass]
struct ModelAction {
    m: Box<dyn model::actions::ActionBuilder>
}


#[pymethods]
impl Model {

    #[new]
    fn new(obj: &PyRawObject, filename: Option<&str>) {
        let model = match filename {
            None => model::new_model(),
            Some(filename) => {
                match io::load_model(filename, None) {
                    Ok(m) => m,
                    Err(e) => model::new_model(),
                }
            }
        };

        obj.init({
            Model { m: model }
        });
    }

    /// Save a model to file
    fn save(&self, filename: &str) {
        io::save_model(self.m.as_ref(), filename, None);
    }

    /// Lock a component of the model
    fn lock(&mut self, name: &str, value: bool) -> bool {
        let model = self.m.as_mut();
        if let Some(uid) = model.component_by_name(name) {
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
        let builder = self.m.as_ref().fixpoints();
        builder.call();
    }

    fn trapspaces(&self) {
        let builder = self.m.as_ref().trapspaces();
        builder.call();
    }

    fn expr(&mut self, repr: &str) {
        let result = io::parse_expr(self.m.as_mut(), repr);
    }
}

#[pymethods]
impl Expr {

    #[staticmethod]
    fn from_bool(val: bool) -> Expr {
        Expr { expr: func::expr::Expr::from_bool(val) }
    }

    #[staticmethod]
    fn from_uid(val: usize) -> Expr {
        Expr { expr: func::expr::Expr::ATOM(val) }
    }

}

#[pyproto]
impl PyObjectProtocol<'_> for Model {

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.m.for_display()))
    }
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

}
