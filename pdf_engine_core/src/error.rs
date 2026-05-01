use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PDF File: Missing %PDF- header")]
    InvalidFileSignature,

    #[error("File is too small to be a valid PDF")]
    FileTooSmall,
}
