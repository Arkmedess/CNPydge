//! Pipeline orchestrator.
//!
//! Connects all stages: mmap -> parse -> normalize -> batch -> write.
//! Uses crossbeam channels for backpressure between stages.
//!
//! Topology:
//! ```text
//! [Reader thread: mmap chunks] -> bounded -> [Parser thread] -> bounded -> [Batcher thread] -> bounded -> [Writer thread]
//! ```

use crate::batch::BatchBuilder;
use crate::channels::{self, Batch, NormalizedRecord};
use crate::metrics::PipelineMetrics;
use crate::mmap::MappedFile;
use crate::normalize::normalize_empresas;
use crate::parser::{parse_line, split_lines};
use crate::postgres::PostgresWriter;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

/// Pipeline configuration.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// PostgreSQL connection string.
    pub dsn: String,
    /// Number of records per batch.
    pub batch_size: usize,
    /// Paths to CSV files to process.
    pub files: Vec<String>,
}

/// Result statistics from a pipeline run.
#[derive(Debug, Clone)]
pub struct ImportStats {
    pub records_processed: u64,
    pub bytes_read: u64,
    pub batches_sent: u64,
    pub records_written: u64,
    pub errors: u64,
    pub elapsed_secs: f64,
    pub records_per_sec: f64,
    pub mb_per_sec: f64,
}

/// Run the full import pipeline.
///
/// This is the main entry point called from Python via PyO3.
/// It spawns threads for each stage and connects them via bounded channels.
///
/// # Errors
/// Returns an error if any stage fails critically.
pub fn run(config: &PipelineConfig) -> Result<ImportStats, PipelineError> {
    let metrics = Arc::new(PipelineMetrics::new());

    // Create channel pairs
    let (raw_tx, raw_rx) = channels::raw_line_channel();
    let (norm_tx, norm_rx) = channels::normalized_record_channel();
    let (batch_tx, batch_rx) = channels::batch_channel();

    // Initialize Postgres writer
    let mut writer = PostgresWriter::connect(&config.dsn, Arc::clone(&metrics))
        .map_err(|e| PipelineError::Database(e.to_string()))?;
    writer
        .initialize()
        .map_err(|e| PipelineError::Database(e.to_string()))?;

    let batch_size = config.batch_size;
    let writer_metrics = Arc::clone(&metrics);

    // Spawn writer thread
    let writer_handle = thread::spawn(move || {
        writer_thread(writer, batch_rx, writer_metrics)
    });

    // Spawn batcher thread
    let batcher_metrics = Arc::clone(&metrics);
    let batcher_handle = thread::spawn(move || {
        batcher_thread(norm_rx, batch_tx, batch_size, batcher_metrics)
    });

    // Spawn parser thread
    let parser_metrics = Arc::clone(&metrics);
    let parser_handle = thread::spawn(move || {
        parser_thread(raw_rx, norm_tx, parser_metrics)
    });

    // Reader: process files sequentially (main thread)
    let reader_metrics = Arc::clone(&metrics);
    for file_path in &config.files {
        let path = Path::new(file_path);
        if let Err(e) = reader_thread(path, &raw_tx, &reader_metrics) {
            eprintln!("Error processing {}: {}", file_path, e);
            metrics.errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    // Signal completion by dropping the sender
    drop(raw_tx);

    // Wait for all threads to finish
    parser_handle
        .join()
        .map_err(|_| PipelineError::Thread("parser panicked".into()))?;
    batcher_handle
        .join()
        .map_err(|_| PipelineError::Thread("batcher panicked".into()))?;
    writer_handle
        .join()
        .map_err(|_| PipelineError::Thread("writer panicked".into()))?;

    let snapshot = metrics.snapshot();
    Ok(ImportStats {
        records_processed: snapshot.records_processed,
        bytes_read: snapshot.bytes_read,
        batches_sent: snapshot.batches_sent,
        records_written: snapshot.records_written,
        errors: snapshot.errors,
        elapsed_secs: snapshot.elapsed_secs,
        records_per_sec: snapshot.records_per_sec,
        mb_per_sec: snapshot.mb_per_sec,
    })
}

/// Reader thread: memory-maps a file and sends raw lines to the parser.
fn reader_thread(
    path: &Path,
    sender: &crossbeam_channel::Sender<Vec<u8>>,
    metrics: &Arc<PipelineMetrics>,
) -> Result<(), PipelineError> {
    let mapped = MappedFile::map(path).map_err(|e| PipelineError::Io(e.to_string()))?;
    let data = mapped.as_bytes();
    metrics.bytes_read.fetch_add(data.len() as u64, Ordering::Relaxed);

    let lines = split_lines(data);
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if sender.send(line.to_vec()).is_err() {
            break;
        }
    }

    Ok(())
}

/// Parser thread: receives raw lines, parses CSV fields, normalizes records.
fn parser_thread(
    receiver: crossbeam_channel::Receiver<Vec<u8>>,
    sender: crossbeam_channel::Sender<NormalizedRecord>,
    metrics: Arc<PipelineMetrics>,
) {
    while let Ok(line) = receiver.recv() {
        if let Some(fields) = parse_line(&line) {
            if let Some(record) = normalize_empresas(&fields) {
                metrics.records_processed.fetch_add(1, Ordering::Relaxed);
                if sender.send(record).is_err() {
                    break;
                }
            } else {
                metrics.errors.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            metrics.errors.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Batcher thread: accumulates records into batches and sends them to the writer.
fn batcher_thread(
    receiver: crossbeam_channel::Receiver<NormalizedRecord>,
    sender: crossbeam_channel::Sender<Batch>,
    batch_size: usize,
    metrics: Arc<PipelineMetrics>,
) {
    let mut builder = BatchBuilder::new(batch_size);

    while let Ok(record) = receiver.recv() {
        if let Some(batch) = builder.push(record) {
            metrics.batches_sent.fetch_add(1, Ordering::Relaxed);
            if sender.send(batch).is_err() {
                break;
            }
        }
    }

    // Flush remaining records
    if !builder.is_empty() {
        let batch = builder.flush();
        metrics.batches_sent.fetch_add(1, Ordering::Relaxed);
        let _ = sender.send(batch);
    }
}

/// Writer thread: receives batches and writes them to Postgres.
fn writer_thread(
    mut writer: PostgresWriter,
    receiver: crossbeam_channel::Receiver<Batch>,
    metrics: Arc<PipelineMetrics>,
) {
    while let Ok(batch) = receiver.recv() {
        match writer.write_batch(&batch) {
            Ok(_count) => {
                // count already added to metrics in write_batch
            }
            Err(e) => {
                eprintln!("Error writing batch: {}", e);
                metrics.errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Pipeline errors.
#[derive(Debug)]
pub enum PipelineError {
    Io(String),
    Database(String),
    Thread(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::Io(msg) => write!(f, "I/O error: {}", msg),
            PipelineError::Database(msg) => write!(f, "Database error: {}", msg),
            PipelineError::Thread(msg) => write!(f, "Thread error: {}", msg),
        }
    }
}

impl std::error::Error for PipelineError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_clone() {
        let config = PipelineConfig {
            dsn: "postgres://localhost/test".into(),
            batch_size: 1000,
            files: vec!["test.csv".into()],
        };
        let cloned = config.clone();
        assert_eq!(cloned.dsn, config.dsn);
        assert_eq!(cloned.batch_size, config.batch_size);
    }
}
