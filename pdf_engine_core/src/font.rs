use std::collections::HashMap;
use crate::error::PdfError;
use crate::lexer::{Lexer, PdfToken};

/// A Character Map (CMap) that translates raw PDF byte codes into actual Unicode strings.
#[derive(Debug, Default)]
pub struct CMap {
    /// Maps a raw PDF character code to a Unicode string
    mapping: HashMap<u32, String>,
}

impl CMap {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    /// Parses a standard `/ToUnicode` CMap stream.
    /// CMaps define ranges using `<bfchar>` and `<bfrange>` blocks.
    pub fn parse(stream_data: &[u8]) -> Result<Self, PdfError> {
        let mut cmap = CMap::new();
        let mut lexer = Lexer::new(stream_data);

        // We do a very naive linear scan of tokens looking for `beginbfchar` and `beginbfrange`
        while let Ok(Some(tok)) = lexer.next_token() {
            if let PdfToken::Keyword(kw) = tok {
                if kw == "beginbfchar" {
                    parse_bfchar(&mut lexer, &mut cmap)?;
                } else if kw == "beginbfrange" {
                    parse_bfrange(&mut lexer, &mut cmap)?;
                }
            }
        }

        Ok(cmap)
    }

    /// Maps a raw byte array (from a PDF string) into a decoded Unicode string.
    pub fn decode_string(&self, bytes: &[u8]) -> String {
        let mut result = String::new();

        // Simple heuristic: if we see 2-byte codes in the CMap, we assume the string is 2-byte encoded.
        // True PDF font parsing requires checking the CIDSystemInfo and FontDescriptor widths.
        // For MVP, we'll try 1-byte, then 2-byte fallback if not found.
        let mut i = 0;
        while i < bytes.len() {
            let code_1 = bytes[i] as u32;
            if let Some(unicode) = self.mapping.get(&code_1) {
                result.push_str(unicode);
                i += 1;
                continue;
            }

            if i + 1 < bytes.len() {
                let code_2 = ((bytes[i] as u32) << 8) | (bytes[i + 1] as u32);
                if let Some(unicode) = self.mapping.get(&code_2) {
                    result.push_str(unicode);
                    i += 2;
                    continue;
                }
            }

            // Fallback: lossy translation
            result.push(bytes[i] as char);
            i += 1;
        }

        result
    }
}

fn parse_bfchar(lexer: &mut Lexer, cmap: &mut CMap) -> Result<(), PdfError> {
    // Format: <src_code> <dst_unicode>
    // Ends with `endbfchar`
    loop {
        let tok = lexer.next_token()?;
        match tok {
            Some(PdfToken::Keyword(kw)) if kw == "endbfchar" => break,
            Some(PdfToken::HexString(src_bytes)) => {
                let dst_tok = lexer.next_token()?;
                if let Some(PdfToken::HexString(dst_bytes)) = dst_tok {
                    let src_code = hex_bytes_to_u32(&src_bytes);
                    let dst_unicode = hex_bytes_to_unicode_string(&dst_bytes);
                    cmap.mapping.insert(src_code, dst_unicode);
                } else {
                    return Err(PdfError::InvalidCMap);
                }
            }
            _ => continue, // ignore whitespace / comments
        }
    }
    Ok(())
}

fn parse_bfrange(lexer: &mut Lexer, cmap: &mut CMap) -> Result<(), PdfError> {
    // Format: <src_start> <src_end> <dst_unicode_start>
    // Or: <src_start> <src_end> [ <dst_1> <dst_2> ... ]
    // Ends with `endbfrange`
    loop {
        let tok = lexer.next_token()?;
        match tok {
            Some(PdfToken::Keyword(kw)) if kw == "endbfrange" => break,
            Some(PdfToken::HexString(start_bytes)) => {
                let end_tok = lexer.next_token()?;
                if let Some(PdfToken::HexString(end_bytes)) = end_tok {
                    let dst_tok = lexer.next_token()?;

                    let start_code = hex_bytes_to_u32(&start_bytes);
                    let end_code = hex_bytes_to_u32(&end_bytes);

                    match dst_tok {
                        Some(PdfToken::HexString(dst_bytes)) => {
                            // Sequential range mapping
                            let mut current_dst = hex_bytes_to_u32(&dst_bytes);
                            for code in start_code..=end_code {
                                cmap.mapping.insert(code, u32_to_unicode_string(current_dst));
                                current_dst += 1;
                            }
                        }
                        Some(PdfToken::ArrayStart) => {
                            // Array of mappings
                            let mut current_code = start_code;
                            while let Some(arr_tok) = lexer.next_token()? {
                                match arr_tok {
                                    PdfToken::ArrayEnd => break,
                                    PdfToken::HexString(dst_bytes) => {
                                        cmap.mapping.insert(current_code, hex_bytes_to_unicode_string(&dst_bytes));
                                        current_code += 1;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => return Err(PdfError::InvalidCMap),
                    }
                } else {
                    return Err(PdfError::InvalidCMap);
                }
            }
            _ => continue,
        }
    }
    Ok(())
}

fn hex_bytes_to_u32(hex_ascii: &[u8]) -> u32 {
    let mut val = 0;
    for &b in hex_ascii {
        let v = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => 0,
        };
        val = (val << 4) | (v as u32);
    }
    val
}

fn hex_bytes_to_unicode_string(hex_ascii: &[u8]) -> String {
    let mut bytes = Vec::new();
    let mut high_nibble = None;
    for &b in hex_ascii {
        let v = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => continue,
        };
        if let Some(h) = high_nibble {
            bytes.push((h << 4) | v);
            high_nibble = None;
        } else {
            high_nibble = Some(v);
        }
    }

    // Convert UTF-16BE (PDF standard for CMap destinations) to UTF-8 Rust String
    if bytes.len() % 2 == 0 {
        let mut u16_chars = Vec::new();
        for i in (0..bytes.len()).step_by(2) {
            u16_chars.push(((bytes[i] as u16) << 8) | (bytes[i + 1] as u16));
        }
        String::from_utf16_lossy(&u16_chars)
    } else {
        // Fallback for weird encodings
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

fn u32_to_unicode_string(val: u32) -> String {
    // Treat the u32 as a UTF-16BE char
    let u16_val = val as u16;
    String::from_utf16_lossy(&[u16_val])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bfchar() {
        let cmap_data = b"
            beginbfchar
            <01> <0041>
            <02> <0042>
            endbfchar
        ";
        let cmap = CMap::parse(cmap_data).unwrap();

        assert_eq!(cmap.mapping.get(&1).unwrap(), "A");
        assert_eq!(cmap.mapping.get(&2).unwrap(), "B");

        let decoded = cmap.decode_string(&[0x01, 0x02]);
        assert_eq!(decoded, "AB");
    }

    #[test]
    fn test_parse_bfrange_sequential() {
        let cmap_data = b"
            beginbfrange
            <01> <05> <0041>
            endbfrange
        ";
        let cmap = CMap::parse(cmap_data).unwrap();

        // 0x01 -> A, 0x02 -> B, 0x03 -> C, 0x04 -> D, 0x05 -> E
        assert_eq!(cmap.mapping.get(&1).unwrap(), "A");
        assert_eq!(cmap.mapping.get(&5).unwrap(), "E");

        let decoded = cmap.decode_string(&[0x01, 0x05]);
        assert_eq!(decoded, "AE");
    }

    #[test]
    fn test_parse_bfrange_array() {
        let cmap_data = b"
            beginbfrange
            <01> <03> [ <0041> <0042> <0043> ]
            endbfrange
        ";
        let cmap = CMap::parse(cmap_data).unwrap();

        assert_eq!(cmap.mapping.get(&1).unwrap(), "A");
        assert_eq!(cmap.mapping.get(&2).unwrap(), "B");
        assert_eq!(cmap.mapping.get(&3).unwrap(), "C");
    }
}

use ttf_parser::Face;

/// Wrapper around a parsed TrueType Font Face.
pub struct TrueTypeFont<'a> {
    pub face: Face<'a>,
}

impl<'a> TrueTypeFont<'a> {
    /// Loads a TrueType font from a raw, decompressed byte stream extracted from a PDF `/FontFile2`.
    pub fn parse(font_data: &'a [u8]) -> Result<Self, PdfError> {
        let face = Face::parse(font_data, 0)
            .map_err(|e| PdfError::InvalidTrueTypeFont(e.to_string()))?;

        Ok(Self { face })
    }

    /// Retrieves the exact mathematical width of a character from the `hmtx` table.
    /// This is crucial for rendering text exactly where Adobe expects it, avoiding overlaps.
    pub fn get_glyph_width(&self, char_code: char) -> Option<u16> {
        let glyph_id = self.face.glyph_index(char_code)?;
        self.face.glyph_hor_advance(glyph_id)
    }
}

#[cfg(test)]
mod ttf_tests {
    use super::*;

    #[test]
    fn test_parse_invalid_truetype() {
        // Pass garbage data to ensure it fails safely without panicking.
        let garbage = b"Not a TrueType Font!";
        let result = TrueTypeFont::parse(garbage);
        assert!(matches!(result, Err(PdfError::InvalidTrueTypeFont(_))));
    }

    // In a real testing environment, we would include a tiny valid `.ttf` file
    // inside a `tests/fixtures/` directory to test `get_glyph_width` successfully.
}
