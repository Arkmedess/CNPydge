//! CNPJ Core -- Rust hot path for the CNPJ Ultra Importer.
//!
//! This crate provides the high-performance processing core:
//! - Memory-mapped file access
//! - CSV parsing (semicolon-delimited, Latin-1)
//! - Record normalization (CNPJ concatenation, field validation)
//! - Parallel pipeline with crossbeam channels
//! - Batch accumulation
//! - PostgreSQL COPY BINARY writer
//!
//! Exposed to Python via PyO3/maturin.

mod batch;
mod channels;
mod filters;
mod metrics;
mod mmap;
mod normalize;
mod parser;
mod pipeline;
mod postgres;
mod simd;

use pyo3::prelude::*;

/// Import statistics returned to Python.
#[pyclass]
#[derive(Debug, Clone)]
pub struct ImportStats {
    #[pyo3(get)]
    pub records_processed: u64,
    #[pyo3(get)]
    pub bytes_read: u64,
    #[pyo3(get)]
    pub batches_sent: u64,
    #[pyo3(get)]
    pub records_written: u64,
    #[pyo3(get)]
    pub errors: u64,
    #[pyo3(get)]
    pub elapsed_secs: f64,
    #[pyo3(get)]
    pub records_per_sec: f64,
    #[pyo3(get)]
    pub mb_per_sec: f64,
}

impl From<pipeline::ImportStats> for ImportStats {
    fn from(stats: pipeline::ImportStats) -> Self {
        Self {
            records_processed: stats.records_processed,
            bytes_read: stats.bytes_read,
            batches_sent: stats.batches_sent,
            records_written: stats.records_written,
            errors: stats.errors,
            elapsed_secs: stats.elapsed_secs,
            records_per_sec: stats.records_per_sec,
            mb_per_sec: stats.mb_per_sec,
        }
    }
}

/// Run the CNPJ import pipeline.
///
/// # Arguments
/// * `files` - List of CSV file paths to process
/// * `dsn` - PostgreSQL connection string
/// * `batch_size` - Number of records per batch (default: 100_000)
///
/// # Returns
/// `ImportStats` with processing metrics.
///
/// # Errors
/// Raises `RuntimeError` if the pipeline fails.
#[pyfunction]
#[pyo3(signature = (files, dsn, batch_size=None))]
fn importar(
    py: Python<'_>,
    files: Vec<String>,
    dsn: String,
    batch_size: Option<usize>,
) -> PyResult<ImportStats> {
    let config = pipeline::PipelineConfig {
        dsn,
        batch_size: batch_size.unwrap_or(100_000),
        files,
    };

    // Release the GIL during the entire pipeline execution.
    // Python is free to do other work while Rust processes the data.
    let stats = py
        .allow_threads(|| pipeline::run(&config))
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

    Ok(stats.into())
}

/// Create the Python module.
#[pymodule]
fn cnpj_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(importar, m)?)?;
    m.add_class::<ImportStats>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_stats_conversion() {
        let pipeline_stats = pipeline::ImportStats {
            records_processed: 1000,
            bytes_read: 1_048_576,
            batches_sent: 10,
            records_written: 1000,
            errors: 0,
            elapsed_secs: 1.5,
            records_per_sec: 666.67,
            mb_per_sec: 0.67,
        };

        let py_stats: ImportStats = pipeline_stats.into();
        assert_eq!(py_stats.records_processed, 1000);
        assert_eq!(py_stats.bytes_read, 1_048_576);
        assert_eq!(py_stats.batches_sent, 10);
    }
}
