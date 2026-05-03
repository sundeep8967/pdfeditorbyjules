use crate::content::ContentOperation;
use crate::error::PdfError;
use crate::graphics::GraphicsStateProcessor;
use crate::object::PdfObject;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Renders a set of PDF ContentOperations onto a rasterized pixel buffer (RGBA).
pub fn render_page_to_pixels(
    width: u32,
    height: u32,
    operations: &[ContentOperation],
) -> Result<Vec<u8>, PdfError> {
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| PdfError::RenderError("Failed to allocate pixel buffer".into()))?;

    // PDF coordinate system originates at Bottom-Left.
    // tiny-skia originates at Top-Left.
    // We apply an initial transform to flip the Y axis.
    let base_transform = Transform::from_row(1.0, 0.0, 0.0, -1.0, 0.0, height as f32);

    let mut path_builder = PathBuilder::new();
    let mut proc = GraphicsStateProcessor::new();

    for op in operations {
        // Track the PDF graphics state (CTM, colors, etc.)
        proc.process_op(op)?;

        // Map PDF drawing operations to Skia paths
        match op.operator.as_str() {
            "m" => {
                // MoveTo
                let x = extract_f32(&op.operands, 0)?;
                let y = extract_f32(&op.operands, 1)?;
                path_builder.move_to(x, y);
            }
            "l" => {
                // LineTo
                let x = extract_f32(&op.operands, 0)?;
                let y = extract_f32(&op.operands, 1)?;
                path_builder.line_to(x, y);
            }
            "c" => {
                // Cubic Bezier (3 control points)
                let x1 = extract_f32(&op.operands, 0)?;
                let y1 = extract_f32(&op.operands, 1)?;
                let x2 = extract_f32(&op.operands, 2)?;
                let y2 = extract_f32(&op.operands, 3)?;
                let x3 = extract_f32(&op.operands, 4)?;
                let y3 = extract_f32(&op.operands, 5)?;
                path_builder.cubic_to(x1, y1, x2, y2, x3, y3);
            }
            "h" => {
                // ClosePath
                path_builder.close();
            }
            "S" | "s" => {
                // Stroke Path
                if let Some(path) = path_builder.finish() {
                    let mut stroke = Stroke::default();
                    stroke.width = proc.current_state.line_width;

                    let mut paint = Paint::default();
                    paint.set_color(crate::color::convert_color(
                        &proc.current_state.stroke_color,
                    ));

                    let ctm = &proc.current_state.ctm;
                    let skia_transform =
                        Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f);

                    pixmap.stroke_path(
                        &path,
                        &paint,
                        &stroke,
                        skia_transform.post_concat(base_transform),
                        None,
                    );
                }
                // PDF spec: drawing operators consume the current path
                path_builder = PathBuilder::new();
            }
            "f" | "F" => {
                // Fill Path
                if let Some(path) = path_builder.finish() {
                    let mut paint = Paint::default();
                    paint.set_color(crate::color::convert_color(&proc.current_state.fill_color));

                    let ctm = &proc.current_state.ctm;
                    let skia_transform =
                        Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f);

                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::Winding,
                        skia_transform.post_concat(base_transform),
                        None,
                    );
                }
                path_builder = PathBuilder::new();
            }
            _ => {
                // Ignore unknown operators (text, image, etc. handled in later tasks)
            }
        }
    }

    // Return the raw RGBA pixels (width * height * 4 bytes)
    Ok(pixmap.data().to_vec())
}

// Helper to extract numbers safely
fn extract_f32(operands: &[PdfObject], idx: usize) -> Result<f32, PdfError> {
    if let Some(obj) = operands.get(idx) {
        match obj {
            PdfObject::Real(r) => Ok(*r),
            PdfObject::Integer(i) => Ok(*i as f32),
            _ => Err(PdfError::InvalidGraphicOperator("Expected Number".into())),
        }
    } else {
        Err(PdfError::InvalidGraphicOperator("Missing Operand".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_path() {
        // Draw a 100x100 square starting at (50, 50)
        let ops = vec![
            ContentOperation {
                operator: "m".into(),
                operands: vec![PdfObject::Integer(50), PdfObject::Integer(50)],
            },
            ContentOperation {
                operator: "l".into(),
                operands: vec![PdfObject::Integer(150), PdfObject::Integer(50)],
            },
            ContentOperation {
                operator: "l".into(),
                operands: vec![PdfObject::Integer(150), PdfObject::Integer(150)],
            },
            ContentOperation {
                operator: "l".into(),
                operands: vec![PdfObject::Integer(50), PdfObject::Integer(150)],
            },
            ContentOperation {
                operator: "h".into(),
                operands: vec![],
            }, // close path
            ContentOperation {
                operator: "f".into(),
                operands: vec![],
            }, // fill
        ];

        let width = 200;
        let height = 200;
        let pixels = render_page_to_pixels(width, height, &ops).unwrap();

        assert_eq!(pixels.len(), (width * height * 4) as usize);

        // Check a pixel inside the square.
        // Original PDF coordinate: (100, 100).
        // Since Y is flipped, in Skia space it's (100, 200-100) = (100, 100).
        let x = 100;
        let y = 100;
        let idx = ((y * width + x) * 4) as usize;

        // It should be filled with black (0, 0, 0, 255)
        assert_eq!(pixels[idx], 0); // R
        assert_eq!(pixels[idx + 1], 0); // G
        assert_eq!(pixels[idx + 2], 0); // B
        assert_eq!(pixels[idx + 3], 255); // A

        // Check a pixel outside the square (e.g. 10, 10)
        let out_idx = ((10 * width + 10) * 4) as usize;
        assert_eq!(pixels[out_idx + 3], 0); // Transparent background
    }
}
