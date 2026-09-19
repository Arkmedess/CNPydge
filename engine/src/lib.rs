#![allow(unsafe_op_in_unsafe_fn, clippy::useless_conversion)]

pub mod converter;
pub mod dominio;
pub mod empresa;
pub mod estabelecimento;
pub mod reader;
pub mod simples;
pub mod socio;
pub mod util;

use converter::TableKind;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Retorna a versão da engine compilada em Rust.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Converte um arquivo CSV do CNPJ para Parquet liberando o GIL do Python.
#[pyfunction]
#[pyo3(signature = (src, dst, kind=None))]
fn to_parquet(py: Python<'_>, src: &str, dst: &str, kind: Option<&str>) -> PyResult<usize> {
    let table_kind: TableKind = match kind {
        Some(k) => TableKind::parse(k).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        None => TableKind::Empresas,
    };

    py.allow_threads(|| {
        converter::to_parquet(src, dst, table_kind)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    })
}

/// Módulo nativo compilado do CNPydge.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(to_parquet, m)?)?;
    Ok(())
}
