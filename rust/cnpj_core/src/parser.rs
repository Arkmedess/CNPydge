//! CSV parser for RFB files.
//!
//! Format: pipe-delimited (`;`), no header, Latin-1 encoding, `\r\n` line endings.
//! The parser works on raw bytes from mmap, splitting fields by the delimiter.
//!
//! Phase 1: naive byte-by-byte parsing.
//! Phase 2: SIMD-accelerated delimiter search (memchr/csv-core).

use crate::channels::ParsedRecord;

/// RFB CSV delimiter (semicolon).
const DELIMITER: u8 = b';';

/// Maximum number of fields per line in RFB files.
/// Empresas has ~30 fields, Estabelecimentos ~40+.
const MAX_FIELDS: usize = 50;

/// Parse a single CSV line (bytes) into a record (vector of field slices).
///
/// The input `line` should NOT include the trailing `\r\n`.
/// Fields are extracted as owned `Vec<u8>` for thread-safety across channels.
///
/// # Errors
/// Returns `None` if the line is empty or malformed.
#[inline]
pub fn parse_line(line: &[u8]) -> Option<ParsedRecord> {
    if line.is_empty() {
        return None;
    }

    let mut fields = Vec::with_capacity(16);
    let mut start = 0;

    for (i, &byte) in line.iter().enumerate() {
        if byte == DELIMITER {
            fields.push(line[start..i].to_vec());
            start = i + 1;
        }
    }
    // Last field (after the final delimiter or the whole line if no delimiter)
    fields.push(line[start..].to_vec());

    if fields.is_empty() {
        return None;
    }

    Some(fields)
}

/// Parse a chunk of bytes (potentially multiple lines) into individual lines.
///
/// Handles both `\r\n` and `\n` line endings.
/// Returns an iterator of line byte slices.
pub fn split_lines(data: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::with_capacity(1024);
    let mut start = 0;

    for i in 0..data.len() {
        if data[i] == b'\n' {
            let end = if i > 0 && data[i - 1] == b'\r' {
                i - 1
            } else {
                i
            };
            if end > start {
                lines.push(&data[start..end]);
            }
            start = i + 1;
        }
    }

    // Handle last line without trailing newline
    if start < data.len() {
        let end = if data.len() > 0 && data[data.len() - 1] == b'\r' {
            data.len() - 1
        } else {
            data.len()
        };
        if end > start {
            lines.push(&data[start..end]);
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_line() {
        let line = b"12345678000195;Empresa Teste;0001;Ativa";
        let fields = parse_line(line).unwrap();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], b"12345678000195");
        assert_eq!(fields[1], b"Empresa Teste");
        assert_eq!(fields[2], b"0001");
        assert_eq!(fields[3], b"Ativa");
    }

    #[test]
    fn test_parse_empty_fields() {
        let line = b"12345678000195;;0001;";
        let fields = parse_line(line).unwrap();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], b"12345678000195");
        assert!(fields[1].is_empty());
        assert_eq!(fields[2], b"0001");
        assert!(fields[3].is_empty());
    }

    #[test]
    fn test_parse_empty_line() {
        assert!(parse_line(b"").is_none());
    }

    #[test]
    fn test_split_lines_crlf() {
        let data = b"line1\r\nline2\r\nline3\r\n";
        let lines = split_lines(data);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], b"line1");
        assert_eq!(lines[1], b"line2");
        assert_eq!(lines[2], b"line3");
    }

    #[test]
    fn test_split_lines_lf() {
        let data = b"line1\nline2\nline3\n";
        let lines = split_lines(data);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], b"line1");
    }

    #[test]
    fn test_split_lines_no_trailing_newline() {
        let data = b"line1\nline2";
        let lines = split_lines(data);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], b"line2");
    }
}
