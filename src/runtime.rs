use std::{
    cell::{RefCell, RefMut},
    sync::mpsc::RecvTimeoutError,
    time::Duration,
};

use deno_core::{JsRuntime, RuntimeOptions};
use pyo3::prelude::*;

use crate::{
    exception::{TimeoutException, V8Exception},
    value::convert_v8_value_to_py_value,
};

#[pyclass(unsendable)]
pub struct DenoRuntime {
    _runtime: RefCell<Option<JsRuntime>>,
}

#[pymethods]
impl DenoRuntime {
    #[new]
    fn new() -> Self {
        DenoRuntime {
            _runtime: RefCell::new(None),
        }
    }

    #[args(timeout_ms = "0")]
    fn eval(&self, py: Python<'_>, source_code: &str, timeout_ms: u64) -> PyResult<PyObject> {
        let mut runtime_ref = self._get_runtime();
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
}

impl DenoRuntime {
    fn _initialize_runtime(&self) {
        let runtime = JsRuntime::new(RuntimeOptions::default());
        self._runtime.replace(Some(runtime));
    }

    fn _teardown_runtime(&self) {
        self._runtime.take();
    }

    fn _get_runtime(&self) -> RefMut<Option<JsRuntime>> {
        {
            let runtime = self._runtime.borrow_mut();
            if runtime.is_some() {
                return runtime;
            }
        }

        self._initialize_runtime();
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
