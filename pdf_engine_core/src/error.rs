use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PdfError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Invalid PDF File: Missing %PDF- header")]
    InvalidFileSignature,

    #[error("File is too small to be a valid PDF")]
    FileTooSmall,

    #[error("Could not find startxref marker in file")]
    MissingStartXref,

    #[error("Invalid offset found after startxref")]
    InvalidStartXrefOffset,

    #[error("Failed to parse XREF table format")]
    InvalidXrefFormat,

    #[error("Lexer encountered unexpected EOF")]
    UnexpectedEof,

    #[error("Lexer encountered invalid syntax: {0}")]
    InvalidSyntax(String),

    #[error("Requested object {0} is marked as Free")]
    ObjectIsFree(u32),

    #[error("Requested object {0} requires stream decompression")]
    ObjectRequiresDecompression(u32),

    #[error("Object {0} not found in XREF table")]
    ObjectNotFound(u32),

    #[error("Malformed indirect object definition for {0}")]
    MalformedIndirectObject(u32),

    #[error("Parser encountered unexpected token")]
    UnexpectedToken,

    #[error("Parser expected Dictionary key to be a Name")]
    ExpectedDictKeyName,

    #[error("Parser encountered unexpected end of stream keyword")]
    UnexpectedEndStream,

    #[error("Parser encountered unexpected end of object keyword")]
    UnexpectedEndObj,

    #[error("Failed to decode stream: {0}")]
    FilterDecodeError(String),

    #[error("Unsupported stream filter: {0}")]
    UnsupportedFilter(String),

    #[error("Missing or invalid Trailer dictionary")]
    InvalidTrailer,

    #[error("Missing /Root entry in Trailer")]
    MissingRoot,

    #[error("Invalid Page Tree structure")]
    InvalidPageTree,

    #[error("Page is missing /Contents entry")]
    MissingPageContents,

    #[error("Invalid Page /Contents structure")]
    InvalidPageContents,

    #[error("Graphics State Stack underflow")]
    GraphicsStackUnderflow,

    #[error("Invalid Graphic Operator: {0}")]
    InvalidGraphicOperator(String),

    #[error("Failed to render page: {0}")]
    RenderError(String),
}

impl From<std::io::Error> for PdfError {
    fn from(err: std::io::Error) -> Self {
        PdfError::Io(err.to_string())
    }
}
