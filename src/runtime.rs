use deno_core::{JsRuntime, RuntimeOptions};
use pyo3::prelude::*;

use crate::{exception::V8Exception, value::convert_v8_value_to_py_value};

#[pyclass(unsendable)]
pub struct DenoRuntime {
    runtime: JsRuntime,
}

#[pymethods]
impl DenoRuntime {
    #[new]
    fn new() -> Self {
        DenoRuntime {
            runtime: JsRuntime::new(RuntimeOptions::default()),
        }
    }

    fn execute_script(
        &mut self,
        py: Python<'_>,
        name: &str,
        source_code: &str,
    ) -> PyResult<PyObject> {
        let result = self
            .runtime
            .execute_script(name, source_code)
            .map_err(|err| V8Exception::new_err(format!("{:?}", err)))?;
        let scope = &mut self.runtime.handle_scope();
        let value = result.open(scope);
        let py_value = convert_v8_value_to_py_value(py, scope, value)?;
        Ok(py_value)
    }

    fn eval(&mut self, py: Python<'_>, source_code: &str) -> PyResult<PyObject> {
        self.execute_script(py, "<eval>", source_code)
    }
}
