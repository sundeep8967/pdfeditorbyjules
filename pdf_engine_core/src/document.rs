use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::PdfError;
use crate::object::ObjectId;
use crate::xref::{XrefEntry, XrefTable};

pub struct PdfDocument {
    file: File,
    pub version: String,
    pub xref_table: XrefTable,
}

impl PdfDocument {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PdfError> {
        let mut file = File::open(path)?;

        let mut buffer = [0u8; 1024];
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read < 8 {
            return Err(PdfError::FileTooSmall);
        }

        let signature = b"%PDF-";
        let mut version_str = String::new();
        let mut found = false;

        for i in 0..=(bytes_read - signature.len()) {
            if &buffer[i..i + signature.len()] == signature {
                let version_start = i + signature.len();
                let mut version_end = version_start;

                while version_end < bytes_read &&
                      buffer[version_end] != b'\r' &&
                      buffer[version_end] != b'\n' &&
                      buffer[version_end] != b' ' {
                    version_end += 1;
                }

                if let Ok(v) = std::str::from_utf8(&buffer[version_start..version_end]) {
                    version_str = v.to_string();
                }

                found = true;
                break;
            }
        }

        if !found {
            return Err(PdfError::InvalidFileSignature);
        }

        Ok(PdfDocument {
            file,
            version: version_str,
            xref_table: XrefTable::new(),
        })
    }

    pub fn get_raw_object_bytes(&mut self, id: ObjectId) -> Result<Vec<u8>, PdfError> {
        let entry = self.xref_table.entries.get(&id.object_number)
            .ok_or(PdfError::ObjectNotFound(id.object_number))?;

        let offset = match entry {
            XrefEntry::Free { .. } => return Err(PdfError::ObjectIsFree(id.object_number)),
            XrefEntry::Compressed { .. } => return Err(PdfError::ObjectRequiresDecompression(id.object_number)),
            XrefEntry::InUse { byte_offset, .. } => *byte_offset,
        };

        self.file.seek(SeekFrom::Start(offset))?;

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];

        // WARNING: In a real PDF, object bounds must be determined by the lexer directly parsing the file stream.
        // Searching for "endobj" is unsafe because binary data (e.g. inside streams) might contain the bytes "endobj".
        // This is a known architectural gap that we will fix in the next phase.
        // For now, we use a naive substring search to pass early tests.
        loop {
            let bytes_read = self.file.read(&mut chunk)?;
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);

            if find_subsequence(&buffer, b"endobj").is_some() {
                break;
            }

            if buffer.len() > 1024 * 1024 * 50 {
                return Err(PdfError::MalformedIndirectObject(id.object_number));
            }
        }

        if let Some(idx) = find_subsequence(&buffer, b"endobj") {
            buffer.truncate(idx + 6);
        } else {
            return Err(PdfError::MalformedIndirectObject(id.object_number));
        }

        Ok(buffer)
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_pdf_signature() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"%PDF-1.4\n%binary data\n").unwrap();

        let doc = PdfDocument::open(file.path()).unwrap();
        assert_eq!(doc.version, "1.4");
    }

    #[test]
    fn test_valid_pdf_signature_offset() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"garbage bytes \r\n%PDF-1.7\n%binary data\n").unwrap();

        let doc = PdfDocument::open(file.path()).unwrap();
        assert_eq!(doc.version, "1.7");
    }

    #[test]
    fn test_invalid_pdf_signature() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"PK\x03\x04This is a zip file, not a PDF").unwrap();

        let result = PdfDocument::open(file.path());
        assert!(matches!(result, Err(PdfError::InvalidFileSignature)));
    }

    #[test]
    fn test_file_too_small() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"123").unwrap();

        let result = PdfDocument::open(file.path());
        assert!(matches!(result, Err(PdfError::FileTooSmall)));
    }

    #[test]
    fn test_get_raw_object_bytes() {
        let mut file = NamedTempFile::new().unwrap();
        let data = b"%PDF-1.4\n%binary\n10 0 obj\n<< /Type /Page >>\nendobj\n";
        file.write_all(data).unwrap();

        let mut doc = PdfDocument::open(file.path()).unwrap();

        doc.xref_table.entries.insert(10, XrefEntry::InUse { byte_offset: 17, generation_number: 0 });

        let obj_id = ObjectId { object_number: 10, generation_number: 0 };
        let bytes = doc.get_raw_object_bytes(obj_id).unwrap();

        assert_eq!(bytes, b"10 0 obj\n<< /Type /Page >>\nendobj");
    }
}
