use std::{mem, slice};

use pyo3::{buffer::*, exceptions::*, prelude::*};

const SEPARATOR: u8 = b':';
const EOF: u8 = b',';

#[inline(always)]
fn bytes_from_buffer<T: Element>(buffer: &PyBuffer<T>) -> PyResult<&[u8]> {
    debug_assert_eq!(mem::size_of::<T>(), 1);

    if !buffer.is_c_contiguous() {
        return Err(PyBufferError::new_err("buffer is not contiguous"));
    }

    unsafe { Ok(slice::from_raw_parts(buffer.buf_ptr() as *const u8, buffer.len_bytes())) }
}

#[pyfunction(signature = (data, /), text_signature = None)]
pub fn dumps<'py>(data: &Bound<'py, PyAny>) -> PyResult<Vec<u8>> {
    let buffer = PyBuffer::<u8>::get(data)?;
    let bytes = bytes_from_buffer(&buffer)?;
    let length = bytes.len();

    let mut vec = Vec::with_capacity({
        match length.checked_ilog10() {
            Some(length_itoa) => length.strict_add(length_itoa as usize).strict_add(3),
            _ => 3,
        }
    });

    vec.extend_from_slice(length.to_string().as_bytes());
    vec.push(SEPARATOR);
    vec.extend_from_slice(bytes);
    vec.push(EOF);

    Ok(vec)
}

#[pyfunction(signature = (data, /), text_signature = None)]
pub fn loads<'py>(data: &Bound<'py, PyAny>) -> PyResult<Vec<u8>> {
    let buffer = PyBuffer::<u8>::get(data)?;
    let bytes = bytes_from_buffer(&buffer)?;

    let separator_idx = bytes
        .iter()
        .position(|&i| i == SEPARATOR)
        .ok_or_else(|| PyValueError::new_err("separator not found"))?;

    let (left, right) = bytes.split_at(separator_idx);

    let length = str::from_utf8(left)
        .map_err(|_| PyUnicodeError::new_err("invalid utf-8 sequence"))?
        .parse::<usize>()
        .map_err(|_| PyValueError::new_err("invalid length"))?;

    match right {
        [_, rest @ .., EOF] if rest.len() == length => Ok(rest.to_vec()),
        _ => Err(PyValueError::new_err("malformed input")),
    }
}

#[pymodule(name = "_netstrings", gil_used = false)]
pub mod netstrings {
    #[pymodule_export]
    pub use super::{dumps, loads};
}

#[cfg(test)]
mod tests {
    use pyo3::{IntoPyObjectExt, exceptions::*, prelude::*};
    use rand::random;

    use super::{dumps, loads};

    #[test]
    fn test_dumps_empty_string() {
        Python::initialize();
        Python::attach(|py| {
            let encoded = dumps(&b"".into_bound_py_any(py).unwrap());
            assert!(encoded.is_ok());

            let actual = encoded.unwrap();
            let expected = b"0:,".to_vec();
            assert_eq!(actual, expected)
        })
    }

    #[test]
    fn test_dumps_standard_string() {
        Python::initialize();
        Python::attach(|py| {
            let encoded = dumps(&b"hello world!".into_bound_py_any(py).unwrap());
            assert!(encoded.is_ok());

            let actual = encoded.unwrap();
            let expected = b"12:hello world!,".to_vec();
            assert_eq!(actual, expected)
        })
    }

    #[test]
    fn test_loads_empty_string() {
        Python::initialize();
        Python::attach(|py| {
            let decoded = loads(&b"0:,".into_bound_py_any(py).unwrap());
            assert!(decoded.is_ok());

            let actual = decoded.unwrap();
            let expected = vec![0u8; 0];
            assert_eq!(actual, expected)
        })
    }

    #[test]
    fn test_loads_standard_string() {
        Python::initialize();
        Python::attach(|py| {
            let decoded = loads(&b"12:hello world!,".into_bound_py_any(py).unwrap());
            assert!(decoded.is_ok());

            let actual = decoded.unwrap();
            let expected = b"hello world!".to_vec();
            assert_eq!(actual, expected)
        })
    }

    #[test]
    fn test_loads_error_separator_not_found() {
        Python::initialize();
        Python::attach(|py| {
            let decoded = loads(&b",data".into_bound_py_any(py).unwrap());
            assert!(decoded.is_err());

            let err = decoded.unwrap_err();
            assert!(err.is_instance_of::<PyValueError>(py));
            assert_eq!(err.value(py).to_string(), "separator not found")
        })
    }

    #[test]
    fn test_loads_error_invalid_sequence() {
        Python::initialize();
        Python::attach(|py| {
            let data: [u8; 10] = [0xF2, 0xE5, 0xF1, 0xF2, 0x3A, 0x74, 0x65, 0x73, 0x74, 0x2C];
            let decoded = loads(&data.into_bound_py_any(py).unwrap());
            assert!(decoded.is_err());

            let err = decoded.unwrap_err();
            assert!(err.is_instance_of::<PyUnicodeError>(py));
            assert_eq!(err.value(py).to_string(), "invalid utf-8 sequence")
        })
    }

    #[test]
    fn test_loads_error_invalid_length() {
        Python::initialize();
        Python::attach(|py| {
            let decoded = loads(&b"test:test,".into_bound_py_any(py).unwrap());
            assert!(decoded.is_err());

            let err = decoded.unwrap_err();
            assert!(err.is_instance_of::<PyValueError>(py));
            assert_eq!(err.value(py).to_string(), "invalid length")
        })
    }

    #[test]
    fn test_loads_error_malformed_input() {
        Python::initialize();
        Python::attach(|py| {
            let decoded = loads(&b"4:test".into_bound_py_any(py).unwrap());
            assert!(decoded.is_err());

            let err = decoded.unwrap_err();
            assert!(err.is_instance_of::<PyValueError>(py));
            assert_eq!(err.value(py).to_string(), "malformed input")
        })
    }

    #[test]
    fn test_round_trip() {
        Python::initialize();
        Python::attach(|py| {
            let expected: [u8; 1_000] = random();

            let encoded = dumps(&expected.into_bound_py_any(py).unwrap());
            assert!(encoded.is_ok());

            let decoded = loads(&encoded.unwrap().into_bound_py_any(py).unwrap());
            assert!(decoded.is_ok());

            let actual = decoded.unwrap();
            assert_eq!(actual, expected)
        })
    }
}
