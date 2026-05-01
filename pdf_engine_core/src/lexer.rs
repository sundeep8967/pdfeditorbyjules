use crate::error::PdfError;

#[derive(Debug, Clone, PartialEq)]
pub enum PdfToken {
    /// e.g. `<<`
    DictStart,
    /// e.g. `>>`
    DictEnd,
    /// e.g. `[`
    ArrayStart,
    /// e.g. `]`
    ArrayEnd,
    /// e.g. `/Name`
    Name(String),
    /// e.g. `(Literal String)`
    StringLiteral(Vec<u8>),
    /// e.g. `<48656C6C6F>`
    HexString(Vec<u8>),
    /// e.g. `123`, `-4.5`
    Number(String),
    /// e.g. `true`, `false`, `null`, `obj`, `endobj`, `stream`, `R`
    Keyword(String),
}

pub struct Lexer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn next_token(&mut self) -> Result<Option<PdfToken>, PdfError> {
        self.skip_whitespace_and_comments();

        if self.pos >= self.data.len() {
            return Ok(None);
        }

        let c = self.data[self.pos];

        match c {
            b'<' => {
                if self.peek() == Some(b'<') {
                    self.pos += 2;
                    Ok(Some(PdfToken::DictStart))
                } else {
                    self.lex_hex_string()
                }
            }
            b'>' => {
                if self.peek() == Some(b'>') {
                    self.pos += 2;
                    Ok(Some(PdfToken::DictEnd))
                } else {
                    Err(PdfError::InvalidSyntax("Unexpected single '>'".into()))
                }
            }
            b'[' => {
                self.pos += 1;
                Ok(Some(PdfToken::ArrayStart))
            }
            b']' => {
                self.pos += 1;
                Ok(Some(PdfToken::ArrayEnd))
            }
            b'/' => self.lex_name(),
            b'(' => self.lex_string_literal(),
            b'+' | b'-' | b'.' | b'0'..=b'9' => self.lex_number(),
            _ => {
                // If it's a regular character, it's a keyword
                if is_regular(c) {
                    self.lex_keyword()
                } else {
                    Err(PdfError::InvalidSyntax(format!("Unexpected character: {}", c as char)))
                }
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos + 1 < self.data.len() {
            Some(self.data[self.pos + 1])
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if is_whitespace(c) {
                self.pos += 1;
            } else if c == b'%' {
                // Comment, skip until EOL
                self.pos += 1;
                while self.pos < self.data.len() && self.data[self.pos] != b'\n' && self.data[self.pos] != b'\r' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn lex_name(&mut self) -> Result<Option<PdfToken>, PdfError> {
        self.pos += 1; // skip '/'
        let start = self.pos;
        while self.pos < self.data.len() && is_regular(self.data[self.pos]) {
            self.pos += 1;
        }
        let name_bytes = &self.data[start..self.pos];
        // Names are UTF-8 in PDF 2.0, but generally standard ascii.
        let name_str = String::from_utf8_lossy(name_bytes).into_owned();
        Ok(Some(PdfToken::Name(name_str)))
    }

    fn lex_number(&mut self) -> Result<Option<PdfToken>, PdfError> {
        let start = self.pos;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b'+' || c == b'-' || c == b'.' || c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = String::from_utf8_lossy(&self.data[start..self.pos]).into_owned();
        Ok(Some(PdfToken::Number(num_str)))
    }

    fn lex_keyword(&mut self) -> Result<Option<PdfToken>, PdfError> {
        let start = self.pos;
        while self.pos < self.data.len() && is_regular(self.data[self.pos]) {
            self.pos += 1;
        }
        let kw_str = String::from_utf8_lossy(&self.data[start..self.pos]).into_owned();
        Ok(Some(PdfToken::Keyword(kw_str)))
    }

    fn lex_hex_string(&mut self) -> Result<Option<PdfToken>, PdfError> {
        self.pos += 1; // skip '<'
        let mut hex_bytes = Vec::new();
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b'>' {
                self.pos += 1;
                break;
            }
            if c.is_ascii_hexdigit() {
                hex_bytes.push(c);
            }
            self.pos += 1;
        }
        // In a real hex parser, we'd convert the hex chars to actual u8 bytes here.
        // For the lexer, returning the raw hex ASCII is fine, the Parser will decode it.
        Ok(Some(PdfToken::HexString(hex_bytes)))
    }

    fn lex_string_literal(&mut self) -> Result<Option<PdfToken>, PdfError> {
        self.pos += 1; // skip '('
        let mut string_bytes = Vec::new();
        let mut open_parens = 1;

        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b'\\' {
                // Escape sequence (we keep it raw for now, parser handles it)
                string_bytes.push(c);
                self.pos += 1;
                if self.pos < self.data.len() {
                    string_bytes.push(self.data[self.pos]);
                }
            } else if c == b'(' {
                open_parens += 1;
                string_bytes.push(c);
            } else if c == b')' {
                open_parens -= 1;
                if open_parens == 0 {
                    self.pos += 1;
                    break;
                }
                string_bytes.push(c);
            } else {
                string_bytes.push(c);
            }
            self.pos += 1;
        }

        if open_parens > 0 {
            return Err(PdfError::UnexpectedEof);
        }

        Ok(Some(PdfToken::StringLiteral(string_bytes)))
    }
}

#[inline]
fn is_whitespace(c: u8) -> bool {
    c == 0 || c == 9 || c == 10 || c == 12 || c == 13 || c == 32
}

#[inline]
fn is_delimiter(c: u8) -> bool {
    c == b'(' || c == b')' || c == b'<' || c == b'>' || c == b'[' || c == b']' || c == b'{' || c == b'}' || c == b'/' || c == b'%'
}

#[inline]
fn is_regular(c: u8) -> bool {
    !is_whitespace(c) && !is_delimiter(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_dict() {
        let data = b"<< /Type /Page /Count 3 >>";
        let mut lexer = Lexer::new(data);

        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::DictStart));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Name("Type".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Name("Page".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Name("Count".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Number("3".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::DictEnd));
        assert_eq!(lexer.next_token().unwrap(), None);
    }

    #[test]
    fn test_lex_strings() {
        let data = b"(Hello World) <48656C6C6F>";
        let mut lexer = Lexer::new(data);

        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::StringLiteral(b"Hello World".to_vec())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::HexString(b"48656C6C6F".to_vec())));
    }

    #[test]
    fn test_lex_array_and_keywords() {
        let data = b"[ 1 0 R true false null ] % comment\n obj";
        let mut lexer = Lexer::new(data);

        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::ArrayStart));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Number("1".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Number("0".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Keyword("R".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Keyword("true".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Keyword("false".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Keyword("null".into())));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::ArrayEnd));
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::Keyword("obj".into())));
    }
}
