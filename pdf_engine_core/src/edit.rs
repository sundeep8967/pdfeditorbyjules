use crate::content::ContentOperation;
use crate::object::PdfObject;

/// Re-serializes a list of ContentOperations back into raw PDF stream bytes.
pub fn serialize_content_operations(operations: &[ContentOperation]) -> Vec<u8> {
    let mut buffer = Vec::new();

    for op in operations {
        for operand in &op.operands {
            buffer.extend_from_slice(&serialize_object(operand));
            buffer.push(b' ');
        }
        buffer.extend_from_slice(op.operator.as_bytes());
        buffer.push(b'\n');
    }

    buffer
}

/// Recursively serializes a PdfObject to its raw PDF byte representation.
pub fn serialize_object(obj: &PdfObject) -> Vec<u8> {
    let mut buf = Vec::new();
    match obj {
        PdfObject::Null => buf.extend_from_slice(b"null"),
        PdfObject::Boolean(b) => {
            if *b {
                buf.extend_from_slice(b"true");
            } else {
                buf.extend_from_slice(b"false");
            }
        }
        PdfObject::Integer(i) => buf.extend_from_slice(i.to_string().as_bytes()),
        PdfObject::Real(f) => buf.extend_from_slice(f.to_string().as_bytes()),
        PdfObject::String(bytes) => {
            // For now, write back as literal string `(...)`
            // Real engines must escape `(`, `)`, and `\` appropriately here.
            buf.push(b'(');
            for &byte in bytes {
                if byte == b'(' || byte == b')' || byte == b'\\' {
                    buf.push(b'\\');
                }
                buf.push(byte);
            }
            buf.push(b')');
        }
        PdfObject::Name(name) => {
            buf.push(b'/');
            buf.extend_from_slice(name.as_bytes());
        }
        PdfObject::Array(arr) => {
            buf.push(b'[');
            buf.push(b' ');
            for item in arr {
                buf.extend_from_slice(&serialize_object(item));
                buf.push(b' ');
            }
            buf.push(b']');
        }
        PdfObject::Dictionary(dict) => {
            buf.extend_from_slice(b"<<\n");
            for (key, val) in &dict.entries {
                buf.push(b'/');
                buf.extend_from_slice(key.as_bytes());
                buf.push(b' ');
                buf.extend_from_slice(&serialize_object(val));
                buf.push(b'\n');
            }
            buf.extend_from_slice(b">>");
        }
        PdfObject::Reference(id) => {
            buf.extend_from_slice(
                format!("{} {} R", id.object_number, id.generation_number).as_bytes(),
            );
        }
        PdfObject::Stream(s) => {
            buf.extend_from_slice(&serialize_object(&PdfObject::Dictionary(s.dict.clone())));
            buf.extend_from_slice(b"\nstream\n");
            buf.extend_from_slice(&s.data);
            buf.extend_from_slice(b"\nendstream");
        }
    }
    buf
}

/// Helper method to find and replace text in a stream of content operations.
/// This is a naive MVP approach to text editing.
pub fn replace_text_in_operations(
    ops: &mut [ContentOperation],
    target: &str,
    replacement: &str,
) -> usize {
    let mut replacements_made = 0;
    let target_bytes = target.as_bytes();

    for op in ops {
        if op.operator == "Tj" {
            if let Some(PdfObject::String(ref mut bytes)) = op.operands.first_mut() {
                if bytes == target_bytes {
                    *bytes = replacement.as_bytes().to_vec();
                    replacements_made += 1;
                }
            }
        }
        // Support TJ arrays
        else if op.operator == "TJ" {
            if let Some(PdfObject::Array(ref mut arr)) = op.operands.first_mut() {
                for item in arr.iter_mut() {
                    if let PdfObject::String(ref mut bytes) = item {
                        if bytes == target_bytes {
                            *bytes = replacement.as_bytes().to_vec();
                            replacements_made += 1;
                        }
                    }
                }
            }
        }
    }

    replacements_made
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_objects() {
        assert_eq!(serialize_object(&PdfObject::Integer(42)), b"42");
        assert_eq!(
            serialize_object(&PdfObject::Name("Font1".into())),
            b"/Font1"
        );
        assert_eq!(
            serialize_object(&PdfObject::String(b"Hello (World)".to_vec())),
            b"(Hello \\(World\\))"
        );
    }

    #[test]
    fn test_replace_text() {
        let mut ops = vec![
            ContentOperation {
                operator: "Tf".into(),
                operands: vec![PdfObject::Name("F1".into()), PdfObject::Integer(12)],
            },
            ContentOperation {
                operator: "Tj".into(),
                operands: vec![PdfObject::String(b"Hello".to_vec())],
            },
        ];

        let count = replace_text_in_operations(&mut ops, "Hello", "Goodbye");
        assert_eq!(count, 1);

        let serialized = serialize_content_operations(&ops);
        let expected = b"/F1 12 Tf\n(Goodbye) Tj\n";
        assert_eq!(serialized, expected);
    }
}
