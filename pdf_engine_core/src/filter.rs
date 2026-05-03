use flate2::read::ZlibDecoder;
use std::io::Read;

use crate::error::PdfError;
use crate::object::{PdfObject, PdfStream};

/// Decodes the raw binary data of a PdfStream based on its /Filter dictionary entry.
pub fn decode_stream(stream: &PdfStream) -> Result<Vec<u8>, PdfError> {
    // If there is no filter, the data is already raw.
    let filter_obj = match stream.dict.get("Filter") {
        Some(f) => f,
        None => return Ok(stream.data.clone()),
    };

    let filters = match filter_obj {
        PdfObject::Name(name) => vec![name.clone()],
        PdfObject::Array(arr) => {
            let mut names = Vec::new();
            for obj in arr {
                if let PdfObject::Name(name) = obj {
                    names.push(name.clone());
                } else {
                    return Err(PdfError::InvalidSyntax(
                        "Filter array contains non-Name object".into(),
                    ));
                }
            }
            names
        }
        _ => {
            return Err(PdfError::InvalidSyntax(
                "Filter must be a Name or Array of Names".into(),
            ))
        }
    };

    let mut current_data = stream.data.clone();

    for filter in filters {
        current_data = apply_decode_filter(&filter, &current_data)?;
    }

    Ok(current_data)
}

fn apply_decode_filter(filter_name: &str, data: &[u8]) -> Result<Vec<u8>, PdfError> {
    match filter_name {
        "FlateDecode" | "Fl" => {
            let mut decoder = ZlibDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| PdfError::FilterDecodeError(e.to_string()))?;
            Ok(decompressed)
        }
        "ASCIIHexDecode" | "AHx" => {
            // Decodes a string of ASCII hex characters into binary data
            let mut decoded = Vec::new();
            let mut high_nibble = None;
            for &byte in data {
                if byte == b'>' {
                    break;
                }
                if byte.is_ascii_whitespace() {
                    continue;
                }

                let val = match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    b'A'..=b'F' => byte - b'A' + 10,
                    _ => {
                        return Err(PdfError::FilterDecodeError(
                            "Invalid character in ASCIIHexDecode".into(),
                        ))
                    }
                };

                if let Some(high) = high_nibble {
                    decoded.push((high << 4) | val);
                    high_nibble = None;
                } else {
                    high_nibble = Some(val);
                }
            }

            // If there's an odd number of hex chars, the spec says to assume a trailing 0
            if let Some(high) = high_nibble {
                decoded.push(high << 4);
            }

            Ok(decoded)
        }
        "ASCII85Decode" | "A85" => Err(PdfError::UnsupportedFilter("ASCII85Decode".into())),
        "LZWDecode" | "LZW" => Err(PdfError::UnsupportedFilter("LZWDecode".into())),
        _ => Err(PdfError::UnsupportedFilter(filter_name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::PdfDictionary;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_no_filter() {
        let stream = PdfStream {
            dict: PdfDictionary::new(),
            data: b"Hello World".to_vec(),
        };
        let decoded = decode_stream(&stream).unwrap();
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn test_flate_decode() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"Compressed PDF stream data!").unwrap();
        let compressed_bytes = encoder.finish().unwrap();

        let mut dict = PdfDictionary::new();
        dict.insert("Filter", PdfObject::Name("FlateDecode".into()));

        let stream = PdfStream {
            dict,
            data: compressed_bytes,
        };

        let decoded = decode_stream(&stream).unwrap();
        assert_eq!(decoded, b"Compressed PDF stream data!");
    }

    #[test]
    fn test_ascii_hex_decode() {
        let mut dict = PdfDictionary::new();
        dict.insert("Filter", PdfObject::Name("ASCIIHexDecode".into()));

        let stream = PdfStream {
            dict,
            data: b"48 656c6c6F>".to_vec(), // "Hello"
        };

        let decoded = decode_stream(&stream).unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_unsupported_filter() {
        let mut dict = PdfDictionary::new();
        dict.insert("Filter", PdfObject::Name("LZWDecode".into()));

        let stream = PdfStream {
            dict,
            data: b"fake lzw data".to_vec(),
        };

        let result = decode_stream(&stream);
        assert!(matches!(result, Err(PdfError::UnsupportedFilter(_))));
    }
}
