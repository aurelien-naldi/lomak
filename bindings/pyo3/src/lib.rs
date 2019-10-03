//#![feature(specialization)]

use pyo3::{PyResult, exceptions};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::class::basic::PyObjectProtocol;

use lomak::func;
use lomak::model;
use lomak::model::actions::*;
use lomak::model::io;


#[pyfunction]
/// Create a new instance
fn new_model() -> Model {
    Model{ m: model::new_model() }
}

#[pyfunction]
/// Create a new instance
fn load_model(filename: &str) -> PyResult<Model> {
    match io::load_model(filename, None) {
        Ok(m) => Ok(Model{m: m}),
        Err(e) => Err(exceptions::ValueError::py_err(format!("{}",e))),
    }
}

#[pyfunction]
/// Create a new instance
fn expr_from_bool(value: bool) -> Expr {
    Expr::from_bool(value)
}



/// Simple python wrapper for our rust code
#[pymodule]
fn lomak(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(new_model))?;
    m.add_wrapped(wrap_pyfunction!(load_model))?;
    m.add_wrapped(wrap_pyfunction!(expr_from_bool))?;
//    m.add_wrapped(wrap_pyfunction!(Model))?;
//    m.add("Model", new_model());
    Ok(())
}

/// Wrap a Logical model as a Python object
#[pyclass]
pub struct Model {
    m: model::LQModelRef
}

/// Python API for Boolean expressions
#[pyclass]
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
    fn new(obj: &PyRawObject) {
        obj.init({
            Model { m: model::new_model() }
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
