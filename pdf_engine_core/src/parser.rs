use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::error::PdfError;
use crate::xref::{XrefEntry, XrefTable};
use std::io::BufRead;
use crate::lexer::Lexer;
use crate::ast_parser::Parser as AstParser;
use crate::object::PdfObject;

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

    let offset: u64 = offset_str.parse().map_err(|_| PdfError::InvalidStartXrefOffset)?;

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

#[cfg(test)]
mod xref_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use crate::object::ObjectId;

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
        assert_eq!(trailer.get("Root").unwrap(), &PdfObject::Reference(ObjectId { object_number: 1, generation_number: 0}));
    }
}
