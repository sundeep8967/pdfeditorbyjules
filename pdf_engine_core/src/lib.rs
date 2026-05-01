pub mod error;
pub mod document;

pub use document::PdfDocument;
pub use error::PdfError;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_pdf_signature() {
        let mut file = NamedTempFile::new().unwrap();
        // PDF signature exactly at start
        file.write_all(b"%PDF-1.4\n%binary data\n").unwrap();

        let doc = PdfDocument::open(file.path()).unwrap();
        assert_eq!(doc.version, "1.4");
    }

    #[test]
    fn test_valid_pdf_signature_offset() {
        let mut file = NamedTempFile::new().unwrap();
        // PDF signature after some garbage bytes (allowed within first 1024 bytes)
        file.write_all(b"garbage bytes \r\n%PDF-1.7\n%binary data\n").unwrap();

        let doc = PdfDocument::open(file.path()).unwrap();
        assert_eq!(doc.version, "1.7");
    }

    #[test]
    fn test_invalid_pdf_signature() {
        let mut file = NamedTempFile::new().unwrap();
        // No PDF signature
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
}
pub mod object;
pub use object::{ObjectId, PdfDictionary, PdfObject, PdfStream};
