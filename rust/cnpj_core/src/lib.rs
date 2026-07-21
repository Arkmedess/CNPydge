//! Ponto de entrada do modulo Python (PyO3). Ver skill: pyo3-bridge.

mod mmap;
mod parser;
mod simd;
mod normalize;
mod channels;
mod batch;
mod postgres;
mod duckdb;
mod arrow;
mod filters;
mod metrics;
mod pipeline;

use pyo3::prelude::*;

#[pymodule]
fn cnpj_core(_py: Python<'_>, _m: &Bound<'_, PyModule>) -> PyResult<()> {
    // TODO: registrar funcoes publicas (ex.: `importar`) -- ver skill pyo3-bridge.
    Ok(())
}
