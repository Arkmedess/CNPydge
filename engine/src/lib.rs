#![allow(unsafe_op_in_unsafe_fn, clippy::useless_conversion)]

pub mod converter;
pub mod empresa;
pub mod reader;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Retorna a versão da engine compilada em Rust.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Converte um arquivo CSV de Empresas para Parquet liberando o GIL do Python.
#[pyfunction]
fn to_parquet(py: Python<'_>, src: &str, dst: &str) -> PyResult<usize> {
    py.allow_threads(|| {
        converter::to_parquet(src, dst).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    })
}

/// Módulo nativo compilado do CNPydge.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(to_parquet, m)?)?;
    Ok(())
}
