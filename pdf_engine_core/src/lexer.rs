use crate::error::PdfError;

#[derive(Debug, Clone, PartialEq)]
pub enum PdfToken {
    DictStart,
    DictEnd,
    ArrayStart,
    ArrayEnd,
    Name(String),
    StringLiteral(Vec<u8>),
    HexString(Vec<u8>),
    Number(String),
    Keyword(String),
    /// Represents raw binary data extracted from a `stream`...`endstream` block.
    StreamData(Vec<u8>),
}

pub struct Lexer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Clone for Lexer<'a> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            pos: self.pos,
        }
    }
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
                if is_regular(c) {
                    let kw = self.lex_keyword()?;

                    if let Some(PdfToken::Keyword(ref s)) = kw {
                        if s == "stream" {
                            // In a real PDF, stream length must be determined by the /Length dictionary entry.
                            // We use a safe sub-sequence search here for MVP testing, but will adapt it to pass length directly soon.
                            return self.lex_stream_data_naive();
                        }
                    }
                    Ok(kw)
                } else {
                    Err(PdfError::InvalidSyntax(format!("Unexpected character: {}", c as char)))
                }
            }
        }
    }

    fn lex_stream_data_naive(&mut self) -> Result<Option<PdfToken>, PdfError> {
        if self.pos < self.data.len() && self.data[self.pos] == b'\r' {
            self.pos += 1;
        }
        if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
            self.pos += 1;
        }

        let start = self.pos;
        let endstream = b"endstream";
        let mut found_idx = None;

        for i in start..self.data.len().saturating_sub(endstream.len() - 1) {
            if &self.data[i..i + endstream.len()] == endstream {
                found_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = found_idx {
            let mut end_data = idx;
            if end_data > start && self.data[end_data - 1] == b'\n' {
                end_data -= 1;
                if end_data > start && self.data[end_data - 1] == b'\r' {
                    end_data -= 1;
                }
            }

            let stream_bytes = self.data[start..end_data].to_vec();
            self.pos = idx + endstream.len();
            Ok(Some(PdfToken::StreamData(stream_bytes)))
        } else {
            Err(PdfError::UnexpectedEof)
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
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.data.len() && is_regular(self.data[self.pos]) {
            self.pos += 1;
        }
        let name_str = String::from_utf8_lossy(&self.data[start..self.pos]).into_owned();
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
        self.pos += 1;
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
        Ok(Some(PdfToken::HexString(hex_bytes)))
    }

    fn lex_string_literal(&mut self) -> Result<Option<PdfToken>, PdfError> {
        self.pos += 1;
        let mut string_bytes = Vec::new();
        let mut open_parens = 1;

        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b'\\' {
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

    #[test]
    fn test_lex_stream() {
        let data = b"stream\nRAW_BINARY_DATA\nendstream";
        let mut lexer = Lexer::new(data);
        assert_eq!(lexer.next_token().unwrap(), Some(PdfToken::StreamData(b"RAW_BINARY_DATA".to_vec())));
    }
}
