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

                let mut is_reference = false;
                let mut is_obj = false;
                let mut ref_gen: u16 = 0;

                // Inspect without holding borrow across self.advance()
                if let Some(PdfToken::Number(ref gen_str)) = self.current_token {
                    let mut lookahead = self.lexer.clone();
                    if let Ok(Some(PdfToken::Keyword(kw))) = lookahead.next_token() {
                        if kw == "R" {
                            if let Ok(g) = gen_str.parse::<u16>() {
                                is_reference = true;
                                ref_gen = g;
                            }
                        } else if kw == "obj" {
                            is_obj = true;
                        }
                    }
                }

                if is_reference {
                    let obj_num = s.parse::<u32>().map_err(|_| PdfError::UnexpectedToken)?;
                    self.advance()?; // Consume gen number
                    self.advance()?; // Consume 'R'

                    return Ok(PdfObject::Reference(ObjectId {
                        object_number: obj_num,
                        generation_number: ref_gen,
                    }));
                } else if is_obj {
                    self.advance()?; // Consume gen number
                    self.advance()?; // Consume 'obj'

                    let inner_obj = self.parse_object()?;

                    let mut has_endobj = false;
                    if let Some(PdfToken::Keyword(ref endkw)) = self.current_token {
                        if endkw == "endobj" {
                            has_endobj = true;
                        }
                    }

                    if has_endobj {
                        self.advance()?; // Consume endobj
                        return Ok(inner_obj);
                    }
                    return Err(PdfError::UnexpectedEndObj);
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
                loop {
                    let is_end = if let Some(PdfToken::ArrayEnd) = self.current_token { true } else { false };
                    if is_end {
                        self.advance()?;
                        break;
                    }
                    if self.current_token.is_none() {
                        return Err(PdfError::UnexpectedEof);
                    }
                    let obj = self.parse_object()?;
                    array.push(obj);
                }
                Ok(PdfObject::Array(array))
            }
            PdfToken::DictStart => {
                self.advance()?;
                let mut dict = PdfDictionary::new();
                loop {
                    let is_end = if let Some(PdfToken::DictEnd) = self.current_token { true } else { false };
                    if is_end {
                        self.advance()?;
                        break;
                    }
                    if self.current_token.is_none() {
                        return Err(PdfError::UnexpectedEof);
                    }

                    let key = if let Some(PdfToken::Name(ref name_str)) = self.current_token {
                        name_str.clone()
                    } else {
                        return Err(PdfError::ExpectedDictKeyName);
                    };

                    self.advance()?; // Consume key
                    let value = self.parse_object()?;
                    dict.insert(key, value);
                }

                let mut stream_data = None;
                if let Some(PdfToken::StreamData(ref data)) = self.current_token {
                    stream_data = Some(data.clone());
                }

                if let Some(data) = stream_data {
                    self.advance()?; // Consume the stream block
                    return Ok(PdfObject::Stream(PdfStream {
                        dict,
                        data,
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
