use std::collections::HashMap;

use deno_core::v8::{Context, GetPropertyNamesArgs, HandleScope, Value};
use pyo3::{
    exceptions::PyValueError,
    types::{PyBool, PyDict, PyFloat, PyList, PyLong, PyTuple, PyUnicode},
    IntoPy, PyAny, PyObject, PyResult, Python,
};
use serde_json::Map;

use crate::exception::V8Exception;

pub fn convert_v8_value_to_py_value(
    py: Python<'_>,
    scope: &mut HandleScope<Context>,
    value: &Value,
) -> PyResult<PyObject> {
    if value.is_number() {
        if value.is_int32() {
            return Ok(value
                .to_int32(scope)
                .ok_or_else(|| V8Exception::new_err("unable to extract int value"))?
                .value()
                .into_py(py));
        } else if value.is_uint32() {
            return Ok(value
                .to_uint32(scope)
                .ok_or_else(|| V8Exception::new_err("unable to extract uint value"))?
                .value()
                .into_py(py));
        }

        return Ok(value
            .to_number(scope)
            .ok_or_else(|| V8Exception::new_err("unable to extract float value"))?
            .value()
            .into_py(py));
    } else if value.is_string() {
        return Ok(value.to_rust_string_lossy(scope).into_py(py));
    } else if value.is_null_or_undefined() {
        return Ok(py.None());
    } else if value.is_boolean() {
        return Ok(value.to_boolean(scope).boolean_value(scope).into_py(py));
    } else if value.is_object() {
        let object = value
            .to_object(scope)
            .ok_or_else(|| V8Exception::new_err("unable to extract object value"))?;

        if value.is_array() {
            let length_key = deno_core::v8::String::new(scope, "length")
                .ok_or_else(|| V8Exception::new_err("unable to create object length key"))?;
            let length = object
                .get(scope, length_key.into())
                .ok_or_else(|| V8Exception::new_err("unable to obtain object length"))?
                .to_uint32(scope)
                .ok_or_else(|| V8Exception::new_err("unable to extract array length value"))?
                .value();

            let array = PyList::empty(py);
            for index in 0..length {
                let array_value = object
                    .get_index(scope, index)
                    .ok_or_else(|| V8Exception::new_err("unable to get array item"))?;
                let converted_value = convert_v8_value_to_py_value(py, scope, &array_value)?;
                array.append(converted_value)?;
            }

            return array.extract();
        } else {
            let properties = object
                .get_property_names(scope, GetPropertyNamesArgs::default())
                .ok_or_else(|| V8Exception::new_err("unable to extract object properties"))?;

            let map = PyDict::new(py);
            for index in 0..properties.length() {
                let property = properties
                    .get_index(scope, index)
                    .ok_or_else(|| V8Exception::new_err("unable to read object property"))?;
                let property_value = object
                    .get(scope, property)
                    .ok_or_else(|| V8Exception::new_err("unable to read object property value"))?;
                let converted_property = convert_v8_value_to_py_value(py, scope, &property)?;
                let converted_value = convert_v8_value_to_py_value(py, scope, &property_value)?;
                map.set_item(converted_property, converted_value)?;
            }

            return map.extract();
        }
    }

    Err(PyValueError::new_err(
        "unable to unpack V8 value, no conversion handler has been defined for this type",
    ))
}

pub fn convert_py_value_to_json(value: &PyAny) -> PyResult<serde_json::Value> {
    if value.is_instance_of::<PyBool>()? {
        return Ok(serde_json::Value::Bool(value.extract::<bool>()?));
    } else if value.is_instance_of::<PyLong>()? {
        return Ok(serde_json::Value::Number(serde_json::Number::from(
            value.extract::<i64>()?,
        )));
    } else if value.is_instance_of::<PyFloat>()? {
        return Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(value.extract::<f64>()?)
                .ok_or_else(|| PyValueError::new_err("unable to serialize float value"))?,
        ));
    } else if value.is_instance_of::<PyUnicode>()? {
        return Ok(serde_json::Value::String(value.extract::<String>()?));
    } else if value.is_instance_of::<PyList>()? {
        let values = value.extract::<Vec<&PyAny>>()?;
        return Ok(serde_json::Value::Array(
            values
                .into_iter()
                .map(convert_py_value_to_json)
                .collect::<PyResult<Vec<serde_json::Value>>>()?,
        ));
    } else if value.is_instance_of::<PyTuple>()? {
        let tuple = value.extract::<&PyTuple>()?;
        return Ok(serde_json::Value::Array(
            tuple
                .into_iter()
                .map(convert_py_value_to_json)
                .collect::<PyResult<Vec<serde_json::Value>>>()?,
        ));
    } else if value.is_instance_of::<PyDict>()? {
        let py_map = value.extract::<HashMap<String, &PyAny>>()?;
        let map = py_map
            .iter()
            .map(|(key, value)| Ok((key.to_string(), convert_py_value_to_json(value)?)))
            .collect::<PyResult<Map<String, serde_json::Value>>>()?;
        return Ok(serde_json::Value::Object(map));
    } else if value.is_none() {
        return Ok(serde_json::Value::Null);
    }

    Err(PyValueError::new_err(
        "unable to serialize Python value, no conversion handler has been defined for this type",
    ))
}
