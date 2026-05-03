use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::error::PdfError;

/// Seeks to the end of a file and reads backwards to find the `startxref` marker.
/// Once found, it parses the byte offset immediately following it.
pub fn find_startxref(file: &mut File) -> Result<u64, PdfError> {
    let file_len = file.metadata()?.len();
    if file_len < 16 {
        return Err(PdfError::FileTooSmall);
    }

    // According to PDF spec, %%EOF should be within the last 1024 bytes.
    // We will read a chunk from the end of the file.
    let chunk_size = std::cmp::min(1024, file_len) as usize;
    let mut buffer = vec![0u8; chunk_size];

    file.seek(SeekFrom::End(-(chunk_size as i64)))?;
    file.read_exact(&mut buffer)?;

    // Search backwards through the buffer for `startxref`
    let startxref_marker = b"startxref";
    let mut marker_index = None;

    // We can iterate backwards through the window
    for i in (0..=(chunk_size - startxref_marker.len())).rev() {
        if &buffer[i..i + startxref_marker.len()] == startxref_marker {
            marker_index = Some(i);
            break;
        }
    }

    let marker_index = marker_index.ok_or(PdfError::MissingStartXref)?;

    // Now we parse the number after `startxref`.
    // It should be followed by whitespace, then ascii digits, then whitespace.
    let after_marker = &buffer[marker_index + startxref_marker.len()..];

    // Skip leading whitespace (spaces, carriage returns, line feeds)
    let mut num_start = 0;
    while num_start < after_marker.len() && is_whitespace(after_marker[num_start]) {
        num_start += 1;
    }

    // Find the end of the number
    let mut num_end = num_start;
    while num_end < after_marker.len() && after_marker[num_end].is_ascii_digit() {
        num_end += 1;
    }

    if num_start == num_end {
        return Err(PdfError::InvalidStartXrefOffset);
    }

    let offset_str = std::str::from_utf8(&after_marker[num_start..num_end])
        .map_err(|_| PdfError::InvalidStartXrefOffset)?;

    let offset: u64 = offset_str.parse().map_err(|_| PdfError::InvalidStartXrefOffset)?;

    Ok(offset)
}

#[inline]
fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\r' || c == b'\n' || c == b'\t'
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_find_startxref_standard() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"dummy data\nstartxref\n12345\n%%EOF\n").unwrap();

        let mut f = file.reopen().unwrap();
        let offset = find_startxref(&mut f).unwrap();
        assert_eq!(offset, 12345);
    }

    #[test]
    fn test_find_startxref_crlf() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"dummy data\r\nstartxref\r\n6789\r\n%%EOF\r\n").unwrap();

        let mut f = file.reopen().unwrap();
        let offset = find_startxref(&mut f).unwrap();
        assert_eq!(offset, 6789);
    }

    #[test]
    fn test_missing_startxref() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"dummy data no marker here %%EOF").unwrap();

        let mut f = file.reopen().unwrap();
        let result = find_startxref(&mut f);
        assert!(matches!(result, Err(PdfError::MissingStartXref)));
    }
}

use crate::xref::{XrefEntry, XrefTable};
use std::io::BufRead;

fn read_line_limited<R: BufRead>(reader: &mut R, line: &mut String, limit: u64) -> Result<usize, PdfError> {
    let n = reader.by_ref().take(limit).read_line(line)?;
    if n as u64 == limit && !line.ends_with('\n') && !line.ends_with('\r') {
        return Err(PdfError::InvalidXrefFormat);
    }
    Ok(n)
}

/// Parses an XREF table from a given byte offset in the file.
pub fn parse_xref_table(file: &mut File, offset: u64) -> Result<XrefTable, PdfError> {
    file.seek(SeekFrom::Start(offset))?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();

    // Read the "xref" keyword
    read_line_limited(&mut reader, &mut line, 1024)?;
    if !line.trim().starts_with("xref") {
        return Err(PdfError::InvalidXrefFormat);
    }
    line.clear();

    let mut table = XrefTable::new();

    // Read subsections
    while read_line_limited(&mut reader, &mut line, 1024)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }
        if trimmed.starts_with("trailer") {
            break;
        }

        // Subsection header: <start_obj_num> <count>
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(PdfError::InvalidXrefFormat);
        }

        let start_obj: u32 = parts[0].parse().map_err(|_| PdfError::InvalidXrefFormat)?;
        let count: u32 = parts[1].parse().map_err(|_| PdfError::InvalidXrefFormat)?;
        line.clear();

        for i in 0..count {
            read_line_limited(&mut reader, &mut line, 1024)?;
            let entry_parts: Vec<&str> = line.trim().split_whitespace().collect();
            if entry_parts.len() < 3 {
                return Err(PdfError::InvalidXrefFormat);
            }

            let num1: u64 = entry_parts[0].parse().map_err(|_| PdfError::InvalidXrefFormat)?;
            let gen: u16 = entry_parts[1].parse().map_err(|_| PdfError::InvalidXrefFormat)?;
            let status = entry_parts[2];

            let entry = match status {
                "n" => XrefEntry::InUse { byte_offset: num1, generation_number: gen },
                "f" => XrefEntry::Free { next_free_object: num1 as u32, generation_number: gen },
                _ => return Err(PdfError::InvalidXrefFormat),
            };

            table.entries.insert(start_obj + i, entry);
            line.clear();
        }
    }

    Ok(table)
}

#[cfg(test)]
mod xref_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_xref_table() {
        let mut file = NamedTempFile::new().unwrap();
        let xref_data = b"xref\n0 4\n0000000000 65535 f \n0000000010 00000 n \n0000000022 00000 n \n0000000045 00000 n \ntrailer\n";
        file.write_all(xref_data).unwrap();

        let mut f = file.reopen().unwrap();
        let table = parse_xref_table(&mut f, 0).unwrap();

        assert_eq!(table.entries.len(), 4);
        assert!(matches!(table.entries.get(&0).unwrap(), XrefEntry::Free { .. }));
        if let XrefEntry::InUse { byte_offset, .. } = table.entries.get(&1).unwrap() {
            assert_eq!(*byte_offset, 10);
        } else {
            panic!("Expected InUse entry");
        }
    }

    #[test]
    fn test_parse_xref_table_oversized_line() {
        let mut file = NamedTempFile::new().unwrap();
        // Construct an XREF table with an extremely long line
        let mut xref_data = b"xref\n".to_vec();
        xref_data.extend(vec![b'A'; 2048]); // 2048 bytes, exceeds 1024 limit
        xref_data.push(b'\n');
        xref_data.extend_from_slice(b"0 1\ntrailer\n");
        file.write_all(&xref_data).unwrap();

        let mut f = file.reopen().unwrap();
        let result = parse_xref_table(&mut f, 0);

        assert!(matches!(result, Err(PdfError::InvalidXrefFormat)));
    }
}
