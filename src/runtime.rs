use std::{
    cell::{RefCell, RefMut},
    sync::mpsc::RecvTimeoutError,
    time::Duration,
};

use deno_core::{JsRuntime, RuntimeOptions};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyTuple},
};

use crate::{
    conversion::{convert_py_value_to_json, convert_v8_value_to_py_value},
    exception::{TimeoutException, V8Exception},
};

#[pyclass(unsendable)]
pub struct DenoRuntime {
    _preload_script: Option<String>,
    _runtime: RefCell<Option<JsRuntime>>,
}

#[pymethods]
impl DenoRuntime {
    #[new]
    #[args(preload_script = "None")]
    fn new(py: Python<'_>, preload_script: Option<&str>) -> Self {
        let _self = DenoRuntime {
            _preload_script: preload_script.map(String::from),
            _runtime: RefCell::new(None),
        };

        _self._reset_runtime(py);
        _self
    }

    #[args(timeout_ms = "0")]
    fn eval(&self, py: Python<'_>, source_code: &str, timeout_ms: u64) -> PyResult<PyObject> {
        let mut runtime_ref = self._get_runtime(py);
        let runtime = runtime_ref.as_mut().expect("unable to obtain runtime");

        if timeout_ms == 0 {
            return self._execute_script(runtime, py, "<eval>", source_code);
        }

        match self._execute_script_with_timeout(runtime, py, "<eval>", source_code, timeout_ms) {
            ExecuteWithTimeoutResult::Result(result) => result,
            ExecuteWithTimeoutResult::TimedOut => {
                // If the execution has timed out, we need to throw away the whole runtime,
                // re-initializing it at the next execution
                runtime_ref.take();
                Err(TimeoutException::new_err("eval call has timed out"))
            }
        }
    }

    #[args(py_args = "*", py_kwargs = "**")]
    fn call(
        &self,
        py: Python<'_>,
        function_name: &str,
        py_args: &PyTuple,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let timeout_ms = py_kwargs
            .and_then(|args| args.get_item("timeout_ms"))
            .and_then(|timeout| timeout.extract::<u64>().ok())
            .unwrap_or(0);

        let json_args: Vec<serde_json::Value> = py_args
            .iter()
            .map(convert_py_value_to_json)
            .collect::<PyResult<Vec<serde_json::Value>>>()?;
        let serialized_args = serde_json::to_string(&json_args).map_err(|err| {
            PyValueError::new_err(format!("unable to serialize function arguments: {:?}", err))
        })?;

        let code = format!("{}.apply(this, {})", function_name, serialized_args);
        self.eval(py, &code, timeout_ms)
    }
}

impl DenoRuntime {
    fn _reset_runtime(&self, py: Python<'_>) {
        let runtime = JsRuntime::new(RuntimeOptions::default());
        self._runtime.replace(Some(runtime));

        if let Some(preload_script) = self._preload_script.as_deref() {
            self.eval(py, preload_script, 0)
                .expect("error while executing preload script");
        }
    }

    fn _get_runtime(&self, py: Python<'_>) -> RefMut<Option<JsRuntime>> {
        {
            let runtime = self._runtime.borrow_mut();
            if runtime.is_some() {
                return runtime;
            }
        }

        self._reset_runtime(py);
        self._runtime.borrow_mut()
    }

    fn _execute_script(
        &self,
        runtime: &mut JsRuntime,
        py: Python<'_>,
        name: &str,
        source_code: &str,
    ) -> PyResult<PyObject> {
        let result = runtime
            .execute_script(name, source_code)
            .map_err(|err| V8Exception::new_err(format!("{:?}", err)))?;
        let mut scope = runtime.handle_scope();
        let value = result.open(&mut scope);
        let py_value = convert_v8_value_to_py_value(py, &mut scope, value)?;
        Ok(py_value)
    }

    fn _execute_script_with_timeout(
        &self,
        runtime: &mut JsRuntime,
        py: Python<'_>,
        name: &str,
        source_code: &str,
        timeout_ms: u64,
    ) -> ExecuteWithTimeoutResult {
        let thread_safe_scope = { runtime.handle_scope().thread_safe_handle() };
        let (timeout_send, timeout_receive) = std::sync::mpsc::channel::<bool>();

        let timeout_thread_handle = std::thread::spawn(move || {
            let timeout_duration = Duration::from_millis(timeout_ms);
            if let Err(RecvTimeoutError::Timeout) = timeout_receive.recv_timeout(timeout_duration) {
                thread_safe_scope.terminate_execution();
                true
            } else {
                false
            }
        });

        let result = self._execute_script(runtime, py, name, source_code);
        let _ = timeout_send.send(true);

        let has_timed_out = timeout_thread_handle
            .join()
            .expect("unable to wait for timeout thread");
        if has_timed_out {
            return ExecuteWithTimeoutResult::TimedOut;
        }

        ExecuteWithTimeoutResult::Result(result)
    }
}

enum ExecuteWithTimeoutResult {
    Result(PyResult<PyObject>),
    TimedOut,
}
