use std::io::Read;
use flate2::read::ZlibDecoder;

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
                    return Err(PdfError::InvalidSyntax("Filter array contains non-Name object".into()));
                }
            }
            names
        }
        _ => return Err(PdfError::InvalidSyntax("Filter must be a Name or Array of Names".into())),
    };

    let mut current_data = stream.data.clone();

    // PDF filters are applied in the order they appear in the array.
    // To decode, we must apply them in the exact order.
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
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| PdfError::FilterDecodeError(e.to_string()))?;
            Ok(decompressed)
        }
        "ASCIIHexDecode" | "AHx" => {
            // Placeholder: Not implemented yet, but common.
            Err(PdfError::UnsupportedFilter("ASCIIHexDecode".into()))
        }
        "ASCII85Decode" | "A85" => {
            // Placeholder: Not implemented yet
            Err(PdfError::UnsupportedFilter("ASCII85Decode".into()))
        }
        "LZWDecode" | "LZW" => {
            Err(PdfError::UnsupportedFilter("LZWDecode".into()))
        }
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
        // Compress some data
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"Compressed PDF stream data!").unwrap();
        let compressed_bytes = encoder.finish().unwrap();

        // Create stream object
        let mut dict = PdfDictionary::new();
        dict.insert("Filter", PdfObject::Name("FlateDecode".into()));

        let stream = PdfStream {
            dict,
            data: compressed_bytes,
        };

        // Decode
        let decoded = decode_stream(&stream).unwrap();
        assert_eq!(decoded, b"Compressed PDF stream data!");
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
