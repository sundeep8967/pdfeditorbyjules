use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::ast_parser::Parser as AstParser;
use crate::error::PdfError;
use crate::lexer::Lexer;
use crate::object::PdfObject;
use crate::xref::{XrefEntry, XrefTable};
use std::io::BufRead;

pub fn find_startxref(file: &mut File) -> Result<u64, PdfError> {
    let file_len = file.metadata()?.len();
    if file_len < 16 {
        return Err(PdfError::FileTooSmall);
    }

    let chunk_size = std::cmp::min(1024, file_len) as usize;
    let mut buffer = vec![0u8; chunk_size];

    file.seek(SeekFrom::End(-(chunk_size as i64)))?;
    file.read_exact(&mut buffer)?;

    let startxref_marker = b"startxref";
    let mut marker_index = None;

    for i in (0..=(chunk_size - startxref_marker.len())).rev() {
        if &buffer[i..i + startxref_marker.len()] == startxref_marker {
            marker_index = Some(i);
            break;
        }
    }

    let marker_index = marker_index.ok_or(PdfError::MissingStartXref)?;
    let after_marker = &buffer[marker_index + startxref_marker.len()..];

    let mut num_start = 0;
    while num_start < after_marker.len() && is_whitespace(after_marker[num_start]) {
        num_start += 1;
    }

    let mut num_end = num_start;
    while num_end < after_marker.len() && after_marker[num_end].is_ascii_digit() {
        num_end += 1;
    }

    if num_start == num_end {
        return Err(PdfError::InvalidStartXrefOffset);
    }

    let offset_str = std::str::from_utf8(&after_marker[num_start..num_end])
        .map_err(|_| PdfError::InvalidStartXrefOffset)?;

    let offset: u64 = offset_str
        .parse()
        .map_err(|_| PdfError::InvalidStartXrefOffset)?;

    Ok(offset)
}

pub fn parse_xref_table(file: &mut File, offset: u64) -> Result<XrefTable, PdfError> {
    file.seek(SeekFrom::Start(offset))?;
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();

    reader.read_line(&mut line)?;
    if !line.trim().starts_with("xref") {
        return Err(PdfError::InvalidXrefFormat);
    }
    line.clear();

    let mut table = XrefTable::new();

    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }
        if trimmed.starts_with("trailer") {
            let mut trailer_data = Vec::new();
            // In case the dictionary is on the same line as `trailer`
            let inline_data = trimmed.trim_start_matches("trailer").trim();
            trailer_data.extend_from_slice(inline_data.as_bytes());

            let mut remaining = Vec::new();
            reader.read_to_end(&mut remaining)?;
            trailer_data.extend_from_slice(&remaining);

            let lexer = Lexer::new(&trailer_data);
            let mut ast_parser = AstParser::new(lexer)?;
            let trailer_obj = ast_parser.parse_object()?;

            if let PdfObject::Dictionary(dict) = trailer_obj {
                table.trailer_dict = Some(dict);
            } else {
                return Err(PdfError::InvalidTrailer);
            }
            break;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(PdfError::InvalidXrefFormat);
        }

        let start_obj: u32 = parts[0].parse().map_err(|_| PdfError::InvalidXrefFormat)?;
        let count: u32 = parts[1].parse().map_err(|_| PdfError::InvalidXrefFormat)?;
        line.clear();

        for i in 0..count {
            reader.read_line(&mut line)?;
            let entry_parts: Vec<&str> = line.split_whitespace().collect();
            if entry_parts.len() < 3 {
                return Err(PdfError::InvalidXrefFormat);
            }

            let num1: u64 = entry_parts[0]
                .parse()
                .map_err(|_| PdfError::InvalidXrefFormat)?;
            let gen: u16 = entry_parts[1]
                .parse()
                .map_err(|_| PdfError::InvalidXrefFormat)?;
            let status = entry_parts[2];

            let entry = match status {
                "n" => XrefEntry::InUse {
                    byte_offset: num1,
                    generation_number: gen,
                },
                "f" => XrefEntry::Free {
                    next_free_object: num1 as u32,
                    generation_number: gen,
                },
                _ => return Err(PdfError::InvalidXrefFormat),
            };

            table.entries.insert(start_obj + i, entry);
            line.clear();
        }
    }

    Ok(table)
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
        file.write_all(b"dummy data\nstartxref\n12345\n%%EOF\n")
            .unwrap();

        let mut f = file.reopen().unwrap();
        let offset = find_startxref(&mut f).unwrap();
        assert_eq!(offset, 12345);
    }

    #[test]
    fn test_find_startxref_crlf() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"dummy data\r\nstartxref\r\n6789\r\n%%EOF\r\n")
            .unwrap();

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

#[cfg(test)]
mod xref_tests {
    use super::*;
    use crate::object::ObjectId;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_xref_table_with_trailer() {
        let mut file = NamedTempFile::new().unwrap();
        let xref_data = b"xref\n0 2\n0000000000 65535 f \n0000000010 00000 n \ntrailer\n<< /Size 2 /Root 1 0 R >>\nstartxref\n100\n%%EOF";
        file.write_all(xref_data).unwrap();

        let mut f = file.reopen().unwrap();
        let table = parse_xref_table(&mut f, 0).unwrap();

        assert_eq!(table.entries.len(), 2);

        // Check Trailer
        let trailer = table.trailer().expect("Trailer should be parsed");
        assert_eq!(trailer.get("Size").unwrap(), &PdfObject::Integer(2));
        assert_eq!(
            trailer.get("Root").unwrap(),
            &PdfObject::Reference(ObjectId {
                object_number: 1,
                generation_number: 0
            })
        );
    }
}

/// A heuristic fallback when the XREF table is missing or irreparably corrupted.
/// Scans the entire file byte-by-byte for `obj` markers to rebuild the XREF table dynamically.
pub fn rebuild_xref_from_linear_scan(file: &mut File) -> Result<XrefTable, PdfError> {
    file.seek(SeekFrom::Start(0))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut table = XrefTable::new();

    // We scan for the pattern `[digits] [digits] obj`
    // Example: `10 0 obj`
    let mut pos = 0;
    while pos < buffer.len() {
        // Find 'obj'
        let obj_marker = b"obj";

        let mut found_idx = None;
        for i in pos..buffer.len().saturating_sub(obj_marker.len()) {
            if &buffer[i..i + 3] == obj_marker {
                // Must be preceded by a space
                if i > 0 && is_whitespace(buffer[i - 1]) {
                    found_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = found_idx {
            // Traverse backwards from idx to parse the `generation` and `object` numbers
            if let Some((obj_num, gen_num, start_offset)) = parse_obj_header_backwards(&buffer, idx)
            {
                // We found a valid object! Insert it.
                table.entries.insert(
                    obj_num,
                    XrefEntry::InUse {
                        byte_offset: start_offset as u64,
                        generation_number: gen_num,
                    },
                );
            }
            pos = idx + 3; // move past 'obj'
        } else {
            break; // No more objects
        }
    }

    // After rebuilding the objects, we still need the Trailer dictionary to know the `/Root`.
    // It is usually near the end of the file. We'll do a naive scan for `trailer` from the end.
    if let Some(trailer_idx) = find_last_subsequence(&buffer, b"trailer") {
        let mut trailer_data = buffer[trailer_idx..].to_vec();

        // Strip out startxref/%%EOF from the end if they exist, to not confuse the parser
        if let Some(startxref_idx) = find_last_subsequence(&trailer_data, b"startxref") {
            trailer_data.truncate(startxref_idx);
        }

        let lexer = Lexer::new(&trailer_data);
        if let Ok(mut ast_parser) = AstParser::new(lexer) {
            // The first token should be `trailer`, which is a keyword. Skip it.
            let _ = ast_parser.parse_object(); // consumes `trailer`
            if let Ok(trailer_obj) = ast_parser.parse_object() {
                if let PdfObject::Dictionary(dict) = trailer_obj {
                    table.trailer_dict = Some(dict);
                }
            }
        }
    }

    if table.trailer_dict.is_none() {
        return Err(PdfError::InvalidTrailer); // Even with heuristics, we need a Root!
    }

    Ok(table)
}

fn parse_obj_header_backwards(buffer: &[u8], obj_idx: usize) -> Option<(u32, u16, usize)> {
    let mut p = obj_idx.saturating_sub(1);

    // Skip whitespace before `obj`
    while p > 0 && is_whitespace(buffer[p]) {
        p -= 1;
    }

    // Parse generation number
    let gen_end = p + 1;
    while p > 0 && buffer[p].is_ascii_digit() {
        p -= 1;
    }
    let gen_start = p + 1;

    if gen_start == gen_end {
        return None;
    }
    let gen_str = std::str::from_utf8(&buffer[gen_start..gen_end]).ok()?;
    let gen_num: u16 = gen_str.parse().ok()?;

    // Skip whitespace before generation number
    while p > 0 && is_whitespace(buffer[p]) {
        p -= 1;
    }

    // Parse object number
    let obj_end = p + 1;
    while p > 0 && buffer[p].is_ascii_digit() {
        p -= 1;
    }
    let obj_start = p + 1;

    if obj_start == obj_end {
        return None;
    }
    let obj_str = std::str::from_utf8(&buffer[obj_start..obj_end]).ok()?;
    let obj_num: u32 = obj_str.parse().ok()?;

    // The start offset of the object is `obj_start`
    Some((obj_num, gen_num, obj_start))
}

fn find_last_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    use crate::object::ObjectId;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_rebuild_xref_from_linear_scan() {
        let mut file = NamedTempFile::new().unwrap();
        // A PDF with a completely missing XREF and startxref
        let pdf_data = b"%PDF-1.4\n\
1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [] >>\nendobj\n\
trailer\n<< /Size 3 /Root 1 0 R >>\n";

        file.write_all(pdf_data).unwrap();

        let mut f = file.reopen().unwrap();
        let table = rebuild_xref_from_linear_scan(&mut f).unwrap();

        // Should have found 1 and 2
        assert_eq!(table.entries.len(), 2);

        if let Some(XrefEntry::InUse { byte_offset, .. }) = table.entries.get(&1) {
            assert_eq!(*byte_offset, 9); // Index of "1 0 obj"
        } else {
            panic!("Object 1 not recovered");
        }

        if let Some(XrefEntry::InUse { byte_offset, .. }) = table.entries.get(&2) {
            assert_eq!(*byte_offset, 58); // Index of "2 0 obj"
        } else {
            panic!("Object 2 not recovered");
        }

        // Should have recovered the trailer
        let trailer = table.trailer().expect("Trailer not recovered");
        assert_eq!(trailer.get("Size").unwrap(), &PdfObject::Integer(3));
    }
}
