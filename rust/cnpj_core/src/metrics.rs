//! Pipeline metrics collection.
//!
//! Per-thread counters avoid false sharing. Aggregated at the end.
//! All metrics are plain counters -- no locks, no atomics in the hot path.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Global pipeline metrics, shared via references.
/// Counters are atomic for thread-safe increments without locks.
pub struct PipelineMetrics {
    pub records_processed: AtomicU64,
    pub bytes_read: AtomicU64,
    pub batches_sent: AtomicU64,
    pub records_written: AtomicU64,
    pub errors: AtomicU64,
    pub start_time: Instant,
}

impl PipelineMetrics {
    pub fn new() -> Self {
        Self {
            records_processed: AtomicU64::new(0),
            bytes_read: AtomicU64::new(0),
            batches_sent: AtomicU64::new(0),
            records_written: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Elapsed time since pipeline start.
    #[inline]
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Records per second.
    pub fn records_per_sec(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.records_processed.load(Ordering::Relaxed) as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Megabytes per second.
    pub fn mb_per_sec(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.bytes_read.load(Ordering::Relaxed) as f64 / elapsed / 1_048_576.0
        } else {
            0.0
        }
    }

    /// Take a snapshot of all counters.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            records_processed: self.records_processed.load(Ordering::Relaxed),
            bytes_read: self.bytes_read.load(Ordering::Relaxed),
            batches_sent: self.batches_sent.load(Ordering::Relaxed),
            records_written: self.records_written.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            elapsed_secs: self.elapsed_secs(),
            records_per_sec: self.records_per_sec(),
            mb_per_sec: self.mb_per_sec(),
        }
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable snapshot of metrics at a point in time.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub records_processed: u64,
    pub bytes_read: u64,
    pub batches_sent: u64,
    pub records_written: u64,
    pub errors: u64,
    pub elapsed_secs: f64,
    pub records_per_sec: f64,
    pub mb_per_sec: f64,
}

impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "records={} ({:.0}/s) | bytes={:.1}MB ({:.1}MB/s) | batches={} | written={} | errors={} | elapsed={:.1}s",
            self.records_processed,
            self.records_per_sec,
            self.bytes_read as f64 / 1_048_576.0,
            self.mb_per_sec,
            self.batches_sent,
            self.records_written,
            self.errors,
            self.elapsed_secs,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_snapshot() {
        let m = PipelineMetrics::new();
        m.records_processed.store(1000, Ordering::Relaxed);
        m.bytes_read.store(1_048_576, Ordering::Relaxed);

        let snap = m.snapshot();
        assert_eq!(snap.records_processed, 1000);
        assert_eq!(snap.bytes_read, 1_048_576);
    }
}
