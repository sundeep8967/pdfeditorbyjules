use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::PdfError;

pub struct PdfDocument {
    // We will hold an open file handle for lazy parsing later
    file: File,
    // The version extracted from the magic bytes, e.g., "1.4"
    pub version: String,
}

impl PdfDocument {
    /// Opens a PDF document and validates the magic bytes signature.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PdfError> {
        let mut file = File::open(path)?;

        // The PDF specification says the magic bytes must be in the first 1024 bytes.
        let mut buffer = [0u8; 1024];
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read < 8 {
            return Err(PdfError::FileTooSmall);
        }

        // Search for %PDF-
        let signature = b"%PDF-";
        let mut version_str = String::new();
        let mut found = false;

        for i in 0..=(bytes_read - signature.len()) {
            if &buffer[i..i + signature.len()] == signature {
                // Extract the version. Typically it's %PDF-1.x\r\n
                // We'll read until we hit a whitespace or end of buffer
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

        // Reset the file cursor to the beginning for future parsing
        file.seek(SeekFrom::Start(0))?;

        Ok(PdfDocument {
            file,
            version: version_str,
        })
    }
}
