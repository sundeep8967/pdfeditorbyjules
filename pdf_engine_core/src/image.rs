use crate::error::PdfError;
use jpeg_decoder::Decoder;
use std::io::Cursor;
use tiny_skia::Pixmap;

/// Decodes raw DCT (JPEG) bytes into a raw RGBA Pixmap for tiny-skia.
pub fn decode_jpeg(data: &[u8]) -> Result<Pixmap, PdfError> {
    let mut decoder = Decoder::new(Cursor::new(data));
    let pixels = decoder
        .decode()
        .map_err(|e| PdfError::FilterDecodeError(format!("JPEG Decode error: {}", e)))?;
    let info = decoder
        .info()
        .ok_or_else(|| PdfError::FilterDecodeError("Failed to get JPEG info".into()))?;

    let mut pixmap = Pixmap::new(info.width as u32, info.height as u32)
        .ok_or_else(|| PdfError::RenderError("Failed to allocate Pixmap for JPEG".into()))?;

    // Convert the decoded RGB/Grayscale pixels to RGBA for tiny-skia
    let mut rgba_pixels = Vec::with_capacity((info.width as usize) * (info.height as usize) * 4);

    match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            for chunk in pixels.chunks_exact(3) {
                rgba_pixels.push(chunk[0]); // R
                rgba_pixels.push(chunk[1]); // G
                rgba_pixels.push(chunk[2]); // B
                rgba_pixels.push(255); // A
            }
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            for chunk in pixels.chunks_exact(4) {
                // Naive CMYK to RGB inversion for preview
                let r = 255 - chunk[0];
                let g = 255 - chunk[1];
                let b = 255 - chunk[2];
                // k is chunk[3]
                rgba_pixels.push(r);
                rgba_pixels.push(g);
                rgba_pixels.push(b);
                rgba_pixels.push(255);
            }
        }
        jpeg_decoder::PixelFormat::L8 => {
            for &gray in &pixels {
                rgba_pixels.push(gray);
                rgba_pixels.push(gray);
                rgba_pixels.push(gray);
                rgba_pixels.push(255);
            }
        }
        _ => {
            return Err(PdfError::FilterDecodeError(
                "Unsupported JPEG pixel format".into(),
            ))
        }
    }

    // Load into Pixmap
    if pixmap.data().len() == rgba_pixels.len() {
        pixmap.data_mut().copy_from_slice(&rgba_pixels);
    }

    Ok(pixmap)
}

/// Parses an XObject Dictionary to figure out width, height, and color space, then delegates to decoders.
pub fn load_image_xobject(
    dict: &crate::object::PdfDictionary,
    raw_stream: &[u8],
) -> Result<Pixmap, PdfError> {
    let filter = match dict.get("Filter") {
        Some(crate::object::PdfObject::Name(n)) => n.as_str(),
        _ => "None",
    };

    match filter {
        "DCTDecode" => decode_jpeg(raw_stream),
        _ => Err(PdfError::UnsupportedFilter(format!(
            "Image Filter: {}",
            filter
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_image_filter() {
        let mut dict = crate::object::PdfDictionary::new();
        dict.insert("Filter", crate::object::PdfObject::Name("JPXDecode".into()));

        let result = load_image_xobject(&dict, b"fake data");
        assert!(matches!(result, Err(PdfError::UnsupportedFilter(_))));
    }
}
