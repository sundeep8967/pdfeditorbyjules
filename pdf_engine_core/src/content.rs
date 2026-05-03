use crate::ast_parser::Parser as AstParser;
use crate::document::PdfDocument;
use crate::error::PdfError;
use crate::filter::decode_stream;
use crate::lexer::{Lexer, PdfToken};
use crate::object::{ObjectId, PdfObject};

/// Represents a single Graphics Operator in a Content Stream.
#[derive(Debug, PartialEq)]
pub struct ContentOperation {
    pub operator: String,
    pub operands: Vec<PdfObject>,
}

pub fn parse_page_contents(
    doc: &mut PdfDocument,
    page_id: ObjectId,
) -> Result<Vec<ContentOperation>, PdfError> {
    let raw_bytes = doc.get_raw_object_bytes(page_id)?;
    let mut page_parser = AstParser::new(Lexer::new(&raw_bytes))?;
    let page_obj = page_parser.parse_object()?;

    let page_dict = match page_obj {
        PdfObject::Dictionary(d) => d,
        _ => return Err(PdfError::InvalidPageTree),
    };

    let contents_obj = match page_dict.get("Contents") {
        Some(c) => c,
        None => return Err(PdfError::MissingPageContents),
    };

    let mut stream_ids = Vec::new();
    match contents_obj {
        PdfObject::Reference(id) => stream_ids.push(*id),
        PdfObject::Array(arr) => {
            for item in arr {
                if let PdfObject::Reference(id) = item {
                    stream_ids.push(*id);
                } else {
                    return Err(PdfError::InvalidPageContents);
                }
            }
        }
        _ => return Err(PdfError::InvalidPageContents),
    }

    let mut full_content_data = Vec::new();
    for stream_id in stream_ids {
        let raw_stream_bytes = doc.get_raw_object_bytes(stream_id)?;
        let mut stream_parser = AstParser::new(Lexer::new(&raw_stream_bytes))?;
        let stream_obj = stream_parser.parse_object()?;

        if let PdfObject::Stream(pdf_stream) = stream_obj {
            let decoded_bytes = decode_stream(&pdf_stream)?;
            full_content_data.extend_from_slice(&decoded_bytes);
            full_content_data.push(b'\n');
        } else {
            return Err(PdfError::InvalidPageContents);
        }
    }

    parse_content_stream(&full_content_data)
}

pub fn parse_content_stream(data: &[u8]) -> Result<Vec<ContentOperation>, PdfError> {
    let mut lexer = Lexer::new(data);
    let mut operations = Vec::new();
    let mut current_operands = Vec::new();

    while let Some(token) = lexer.next_token()? {
        match token {
            PdfToken::Keyword(kw) => {
                if kw == "true" {
                    current_operands.push(PdfObject::Boolean(true));
                } else if kw == "false" {
                    current_operands.push(PdfObject::Boolean(false));
                } else if kw == "null" {
                    current_operands.push(PdfObject::Null);
                } else {
                    operations.push(ContentOperation {
                        operator: kw,
                        operands: std::mem::take(&mut current_operands),
                    });
                }
            }
            PdfToken::Number(n) => {
                if let Ok(int_val) = n.parse::<i32>() {
                    current_operands.push(PdfObject::Integer(int_val));
                } else if let Ok(float_val) = n.parse::<f32>() {
                    current_operands.push(PdfObject::Real(float_val));
                }
            }
            PdfToken::StringLiteral(bytes) | PdfToken::HexString(bytes) => {
                current_operands.push(PdfObject::String(bytes));
            }
            PdfToken::Name(n) => {
                current_operands.push(PdfObject::Name(n));
            }
            PdfToken::ArrayStart => {
                let mut arr = Vec::new();
                while let Some(inner_tok) = lexer.next_token()? {
                    match inner_tok {
                        PdfToken::ArrayEnd => break,
                        PdfToken::Number(n) => {
                            if let Ok(v) = n.parse::<i32>() { arr.push(PdfObject::Integer(v)); }
                            else if let Ok(v) = n.parse::<f32>() { arr.push(PdfObject::Real(v)); }
                        }
                        PdfToken::StringLiteral(b) | PdfToken::HexString(b) => arr.push(PdfObject::String(b)),
                        PdfToken::Name(n) => arr.push(PdfObject::Name(n)),
                        _ => return Err(PdfError::InvalidSyntax("Complex nested objects in content stream array not fully supported yet".into())),
                    }
                }
                current_operands.push(PdfObject::Array(arr));
            }
            PdfToken::DictStart => {
                return Err(PdfError::InvalidSyntax(
                    "Inline Image Dictionaries (EI) not yet supported".into(),
                ));
            }
            _ => return Err(PdfError::UnexpectedToken),
        }
    }

    Ok(operations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content_stream() {
        let stream_data = b"0.1 0 0 0.1 100 200 cm\n/F1 12 Tf\n(Hello World) Tj\n";
        let ops = parse_content_stream(stream_data).unwrap();

        assert_eq!(ops.len(), 3);

        assert_eq!(ops[0].operator, "cm");
        assert_eq!(ops[0].operands.len(), 6);
        assert_eq!(ops[0].operands[0], PdfObject::Real(0.1));
        assert_eq!(ops[0].operands[4], PdfObject::Integer(100));

        assert_eq!(ops[1].operator, "Tf");
        assert_eq!(ops[1].operands.len(), 2);
        assert_eq!(ops[1].operands[0], PdfObject::Name("F1".to_string()));
        assert_eq!(ops[1].operands[1], PdfObject::Integer(12));

        assert_eq!(ops[2].operator, "Tj");
        assert_eq!(ops[2].operands.len(), 1);
        assert_eq!(
            ops[2].operands[0],
            PdfObject::String(b"Hello World".to_vec())
        );
    }
}
