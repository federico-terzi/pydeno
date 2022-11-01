use deno_core::{JsRuntime, RuntimeOptions};
use pyo3::prelude::*;

#[pyclass(unsendable)]
struct DenoRuntime {
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

    fn execute_script(&mut self, name: &str, source_code: &str) -> PyResult<String> {
        let result = self.runtime.execute_script(name, source_code).unwrap();
        let scope = &mut self.runtime.handle_scope();
        let value = result.open(scope);
        let string = value.to_string(scope).unwrap();
        let string = string.to_rust_string_lossy(scope);
        Ok(string)
    }
}

#[pymodule]
fn pydeno(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DenoRuntime>()?;
    Ok(())
}
