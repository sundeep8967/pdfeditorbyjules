use std::collections::HashMap;

/// Represents a reference to an indirect object in a PDF file (e.g., `10 0 R`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId {
    pub object_number: u32,
    pub generation_number: u16,
}

/// Represents a PDF Dictionary.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfDictionary {
    // A PDF Dictionary is a mapping of PDF Names to PDF Objects.
    pub entries: HashMap<String, PdfObject>,
}

impl PdfDictionary {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: PdfObject) {
        self.entries.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.get(key)
    }
}

/// Represents a PDF Stream (Dictionary + bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    pub dict: PdfDictionary,
    pub data: Vec<u8>,
}

/// The core abstract syntax tree representing any PDF primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    /// The null object.
    Null,
    /// A boolean value (`true` or `false`).
    Boolean(bool),
    /// An integer value.
    Integer(i32),
    /// A real (floating-point) value.
    Real(f32),
    /// A string literal (e.g., `(Hello World)` or `<48656C6C6F>`).
    String(Vec<u8>),
    /// A name object (e.g., `/Type`).
    Name(String),
    /// An array of objects.
    Array(Vec<PdfObject>),
    /// A dictionary of objects.
    Dictionary(PdfDictionary),
    /// A stream of bytes along with a dictionary describing it.
    Stream(PdfStream),
    /// A reference to an indirect object (e.g., `12 0 R`).
    Reference(ObjectId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_dictionary() {
        let mut dict = PdfDictionary::new();
        dict.insert("Type", PdfObject::Name("Page".to_string()));
        dict.insert("MediaBox", PdfObject::Array(vec![
            PdfObject::Integer(0),
            PdfObject::Integer(0),
            PdfObject::Integer(612),
            PdfObject::Integer(792),
        ]));

        assert_eq!(
            dict.get("Type"),
            Some(&PdfObject::Name("Page".to_string()))
        );

        if let Some(PdfObject::Array(arr)) = dict.get("MediaBox") {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[2], PdfObject::Integer(612));
        } else {
            panic!("MediaBox was not an array");
        }
    }

    #[test]
    fn test_object_sizes() {
        // It's generally good practice to keep the core AST enum small.
        // We ensure we haven't accidentally made it massive.
        // On 64-bit platforms, an enum's size is max(variant_sizes) + discriminant padding.
        // A Vec is 24 bytes, String is 24 bytes, HashMap is larger.
        // We aren't strictly enforcing a specific size here, just verifying it compiles and is reasonable.
        assert!(std::mem::size_of::<PdfObject>() <= 80);
    }
}
