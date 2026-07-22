//! Memory-mapped file access for RFB CSV files.
//!
//! Uses `memmap2` for zero-copy file reading. The OS manages page faults and
//! disk cache, keeping RAM constant regardless of file size.

use memmap2::{Advice, Mmap};
use std::fs::File;
use std::io;
use std::path::Path;

/// Memory-mapped file handle.
///
/// The mmap is valid for the lifetime of this struct. The underlying file
/// must not be modified by another process during import (guaranteed by the
/// download/cache lock).
pub struct MappedFile {
    mmap: Mmap,
    len: usize,
}

impl MappedFile {
    /// Map a file into memory for sequential reading.
    ///
    /// # Errors
    /// Returns `io::Error` if the file cannot be opened or mapped.
    pub fn map(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len() as usize;

        // SAFETY: the file is not modified by another process during import.
        // This is guaranteed by the download/cache lock mechanism.
        let mmap = unsafe { Mmap::map(&file)? };

        // Hint sequential access -- helps the OS prefetch pages.
        mmap.advise(Advice::Sequential)?;

        Ok(Self { mmap, len })
    }

    /// Return the mapped bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.mmap
    }

    /// Return the file size in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the file is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_map_file() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"hello world").unwrap();
        tmp.flush().unwrap();

        let mapped = MappedFile::map(tmp.path()).unwrap();
        assert_eq!(mapped.len(), 11);
        assert_eq!(mapped.as_bytes(), b"hello world");
    }

    #[test]
    fn test_empty_file() {
        let tmp = NamedTempFile::new().unwrap();
        let mapped = MappedFile::map(tmp.path()).unwrap();
        assert!(mapped.is_empty());
        assert_eq!(mapped.len(), 0);
    }
}
