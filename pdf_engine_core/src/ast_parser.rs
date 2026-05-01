use crate::error::PdfError;
use crate::lexer::{Lexer, PdfToken};
use crate::object::{ObjectId, PdfDictionary, PdfObject, PdfStream};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<PdfToken>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self, PdfError> {
        let first_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token: first_token,
        })
    }

    fn advance(&mut self) -> Result<(), PdfError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    pub fn parse_object(&mut self) -> Result<PdfObject, PdfError> {
        let token = match self.current_token.take() {
            Some(t) => t,
            None => return Err(PdfError::UnexpectedEof),
        };

        match token {
            PdfToken::Name(s) => {
                self.advance()?;
                Ok(PdfObject::Name(s))
            }
            PdfToken::StringLiteral(bytes) => {
                self.advance()?;
                Ok(PdfObject::String(bytes))
            }
            PdfToken::HexString(bytes) => {
                self.advance()?;
                Ok(PdfObject::String(bytes))
            }
            PdfToken::Number(s) => {
                self.advance()?;

                if let Some(PdfToken::Number(ref gen_str)) = self.current_token {
                    let mut lookahead = self.lexer.clone();
                    if let Ok(Some(PdfToken::Keyword(kw))) = lookahead.next_token() {
                        if kw == "R" {
                            let obj_num = s.parse::<u32>().map_err(|_| PdfError::UnexpectedToken)?;
                            let gen_num = gen_str.parse::<u16>().map_err(|_| PdfError::UnexpectedToken)?;

                            self.advance()?; // Consume gen number
                            self.advance()?; // Consume 'R'

                            return Ok(PdfObject::Reference(ObjectId {
                                object_number: obj_num,
                                generation_number: gen_num,
                            }));
                        } else if kw == "obj" {
                            // indirect object definition `10 0 obj`
                            // We consume `gen` and `obj`
                            self.advance()?;
                            self.advance()?;

                            // Parse the actual object inside
                            let inner_obj = self.parse_object()?;

                            // It MUST be followed by `endobj`
                            if let Some(PdfToken::Keyword(endkw)) = &self.current_token {
                                if endkw == "endobj" {
                                    self.advance()?; // Consume endobj
                                    return Ok(inner_obj);
                                }
                            }
                            return Err(PdfError::UnexpectedEndObj);
                        }
                    }
                }

                if let Ok(int_val) = s.parse::<i32>() {
                    return Ok(PdfObject::Integer(int_val));
                }

                if let Ok(float_val) = s.parse::<f32>() {
                    return Ok(PdfObject::Real(float_val));
                }

                Err(PdfError::InvalidSyntax(format!("Invalid number format: {}", s)))
            }
            PdfToken::Keyword(kw) => {
                self.advance()?;
                match kw.as_str() {
                    "true" => Ok(PdfObject::Boolean(true)),
                    "false" => Ok(PdfObject::Boolean(false)),
                    "null" => Ok(PdfObject::Null),
                    _ => Err(PdfError::UnexpectedToken),
                }
            }
            PdfToken::ArrayStart => {
                self.advance()?;
                let mut array = Vec::new();
                while let Some(tok) = &self.current_token {
                    if *tok == PdfToken::ArrayEnd {
                        self.advance()?;
                        break;
                    }
                    let obj = self.parse_object()?;
                    array.push(obj);
                }
                Ok(PdfObject::Array(array))
            }
            PdfToken::DictStart => {
                self.advance()?;
                let mut dict = PdfDictionary::new();
                while let Some(tok) = &self.current_token {
                    if *tok == PdfToken::DictEnd {
                        self.advance()?;
                        break;
                    }

                    let key = if let PdfToken::Name(name_str) = tok {
                        name_str.clone()
                    } else {
                        return Err(PdfError::ExpectedDictKeyName);
                    };

                    self.advance()?; // Consume key
                    let value = self.parse_object()?;
                    dict.insert(key, value);
                }

                // PDF Streams are Dictionaries followed immediately by the `stream` token
                if let Some(PdfToken::StreamData(data)) = &self.current_token {
                    let stream_data = data.clone();
                    self.advance()?; // Consume the stream block
                    return Ok(PdfObject::Stream(PdfStream {
                        dict,
                        data: stream_data,
                    }));
                }

                Ok(PdfObject::Dictionary(dict))
            }
            _ => Err(PdfError::UnexpectedToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primitives() {
        let lexer = Lexer::new(b"/Name (String) 123 -4.5 true false null");
        let mut parser = Parser::new(lexer).unwrap();

        assert_eq!(parser.parse_object().unwrap(), PdfObject::Name("Name".into()));
        assert_eq!(parser.parse_object().unwrap(), PdfObject::String(b"String".to_vec()));
        assert_eq!(parser.parse_object().unwrap(), PdfObject::Integer(123));
        assert_eq!(parser.parse_object().unwrap(), PdfObject::Real(-4.5));
        assert_eq!(parser.parse_object().unwrap(), PdfObject::Boolean(true));
        assert_eq!(parser.parse_object().unwrap(), PdfObject::Boolean(false));
        assert_eq!(parser.parse_object().unwrap(), PdfObject::Null);
    }

    #[test]
    fn test_parse_reference() {
        let lexer = Lexer::new(b"10 0 R");
        let mut parser = Parser::new(lexer).unwrap();

        assert_eq!(parser.parse_object().unwrap(), PdfObject::Reference(ObjectId {
            object_number: 10,
            generation_number: 0,
        }));
    }

    #[test]
    fn test_parse_indirect_object() {
        let lexer = Lexer::new(b"10 0 obj\n/Page\nendobj");
        let mut parser = Parser::new(lexer).unwrap();

        // The parser should unwrap the `obj...endobj` and just return the inner Name
        assert_eq!(parser.parse_object().unwrap(), PdfObject::Name("Page".into()));
    }

    #[test]
    fn test_parse_stream() {
        let lexer = Lexer::new(b"<< /Length 11 >>\nstream\nHELLO WORLD\nendstream");
        let mut parser = Parser::new(lexer).unwrap();

        let obj = parser.parse_object().unwrap();
        if let PdfObject::Stream(s) = obj {
            assert_eq!(s.dict.get("Length").unwrap(), &PdfObject::Integer(11));
            assert_eq!(s.data, b"HELLO WORLD");
        } else {
            panic!("Expected Stream");
        }
    }

    #[test]
    fn test_parse_array() {
        let lexer = Lexer::new(b"[ 1 2 /Three ]");
        let mut parser = Parser::new(lexer).unwrap();

        let obj = parser.parse_object().unwrap();
        if let PdfObject::Array(arr) = obj {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[2], PdfObject::Name("Three".into()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_dictionary() {
        let lexer = Lexer::new(b"<< /Type /Page /Count 3 >>");
        let mut parser = Parser::new(lexer).unwrap();

        let obj = parser.parse_object().unwrap();
        if let PdfObject::Dictionary(dict) = obj {
            assert_eq!(dict.get("Type").unwrap(), &PdfObject::Name("Page".into()));
            assert_eq!(dict.get("Count").unwrap(), &PdfObject::Integer(3));
        } else {
            panic!("Expected dictionary");
        }
    }
}
