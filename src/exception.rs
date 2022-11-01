use pyo3::{create_exception, exceptions::PyException};

create_exception!(pydeno, V8Exception, PyException);
create_exception!(pydeno, TimeoutException, PyException);
