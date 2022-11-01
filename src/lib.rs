use pyo3::prelude::*;

mod conversion;
mod exception;
mod runtime;

#[pymodule]
fn pydeno(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<runtime::DenoRuntime>()?;
    m.add("V8Exception", _py.get_type::<exception::V8Exception>())?;
    m.add(
        "TimeoutException",
        _py.get_type::<exception::TimeoutException>(),
    )?;

    Ok(())
}
