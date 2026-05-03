use crate::content::ContentOperation;
use crate::error::PdfError;
use crate::graphics::ColorSpace;
use crate::graphics::GraphicsStateProcessor;
use crate::object::PdfObject;
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Mask, Paint, PathBuilder, Pixmap, Point,
    SpreadMode, Stroke, Transform,
};

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

    // Clip mask state tracking
    let mut current_clip: Option<tiny_skia::Mask> = None;
    let mut clip_stack: Vec<Option<tiny_skia::Mask>> = Vec::new();
    let mut pending_clip_rule: Option<FillRule> = None;

    for op in operations {
        // Handle Save/Restore state for clipping stack before calling process_op
        if op.operator == "q" {
            clip_stack.push(current_clip.clone());
        } else if op.operator == "Q" {
            if let Some(saved_clip) = clip_stack.pop() {
                current_clip = saved_clip;
            }
        }

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
            "W" => {
                // Clip NonZero Winding Rule
                pending_clip_rule = Some(FillRule::Winding);
            }
            "W*" => {
                // Clip EvenOdd Rule
                pending_clip_rule = Some(FillRule::EvenOdd);
            }
            "n" => {
                // End path without filling or stroking (often used for clipping)
                if let Some(rule) = pending_clip_rule.take() {
                    if let Some(path) = path_builder.clone().finish() {
                        let ctm = &proc.current_state.ctm;
                        let skia_transform =
                            Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f);
                        let final_transform = skia_transform.post_concat(base_transform);

                        if let Some(transformed_path) = path.transform(final_transform) {
                            if current_clip.is_none() {
                                current_clip = Mask::new(width, height);
                                if let Some(mask) = current_clip.as_mut() {
                                    mask.fill_path(
                                        &transformed_path,
                                        rule,
                                        false,
                                        Transform::identity(),
                                    );
                                }
                            } else if let Some(mask) = current_clip.as_mut() {
                                mask.intersect_path(
                                    &transformed_path,
                                    rule,
                                    false,
                                    Transform::identity(),
                                );
                            }
                        }
                    }
                }
                path_builder = PathBuilder::new();
            }
            "S" | "s" => {
                // Stroke Path
                let rule = pending_clip_rule.take();

                if let Some(path) = path_builder.finish() {
                    let ctm = &proc.current_state.ctm;
                    let skia_transform =
                        Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f);
                    let final_transform = skia_transform.post_concat(base_transform);

                    if let Some(rule) = rule {
                        if let Some(transformed_path) = path.clone().transform(final_transform) {
                            if current_clip.is_none() {
                                current_clip = tiny_skia::Mask::new(width, height);
                                if let Some(mask) = current_clip.as_mut() {
                                    mask.fill_path(
                                        &transformed_path,
                                        rule,
                                        false,
                                        Transform::identity(),
                                    );
                                }
                            } else if let Some(mask) = current_clip.as_mut() {
                                mask.intersect_path(
                                    &transformed_path,
                                    rule,
                                    false,
                                    Transform::identity(),
                                );
                            }
                        }
                    }

                    let mut stroke = Stroke::default();
                    stroke.width = proc.current_state.line_width;

                    let mut paint = Paint::default();
                    paint.set_color(crate::color::convert_color(
                        &proc.current_state.stroke_color,
                        proc.current_state.stroke_alpha,
                    ));

                    if let Some(mask) = current_clip.as_ref() {
                        let mut temp_pixmap = Pixmap::new(width, height).unwrap();
                        temp_pixmap.stroke_path(&path, &paint, &stroke, final_transform, None);
                        temp_pixmap.apply_mask(mask);
                        pixmap.draw_pixmap(0, 0, temp_pixmap.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
                    } else {
                        pixmap.stroke_path(&path, &paint, &stroke, final_transform, None);
                    }
                }
                // PDF spec: drawing operators consume the current path
                path_builder = PathBuilder::new();
            }
            "f" | "F" => {
                // Fill Path
                let rule = pending_clip_rule.take();

                if let Some(path) = path_builder.finish() {
                    let ctm = &proc.current_state.ctm;
                    let skia_transform =
                        Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f);
                    let final_transform = skia_transform.post_concat(base_transform);

                    if let Some(rule) = rule {
                        if let Some(transformed_path) = path.clone().transform(final_transform) {
                            if current_clip.is_none() {
                                current_clip = tiny_skia::Mask::new(width, height);
                                if let Some(mask) = current_clip.as_mut() {
                                    mask.fill_path(
                                        &transformed_path,
                                        rule,
                                        false,
                                        Transform::identity(),
                                    );
                                }
                            } else if let Some(mask) = current_clip.as_mut() {
                                mask.intersect_path(
                                    &transformed_path,
                                    rule,
                                    false,
                                    Transform::identity(),
                                );
                            }
                        }
                    }

                    let mut paint = Paint::default();

                    if let ColorSpace::LinearGradient {
                        start,
                        end,
                        start_color,
                        end_color,
                    } = proc.current_state.fill_color
                    {
                        let c1 = Color::from_rgba(
                            start_color.0,
                            start_color.1,
                            start_color.2,
                            proc.current_state.fill_alpha,
                        )
                        .unwrap_or(Color::BLACK);
                        let c2 = Color::from_rgba(
                            end_color.0,
                            end_color.1,
                            end_color.2,
                            proc.current_state.fill_alpha,
                        )
                        .unwrap_or(Color::BLACK);

                        let gradient = LinearGradient::new(
                            Point::from_xy(start.0, start.1),
                            Point::from_xy(end.0, end.1),
                            vec![GradientStop::new(0.0, c1), GradientStop::new(1.0, c2)],
                            SpreadMode::Pad,
                            Transform::identity(),
                        )
                        .unwrap_or_else(|| {
                            // Fallback
                            LinearGradient::new(
                                Point::from_xy(0.0, 0.0),
                                Point::from_xy(1.0, 1.0),
                                vec![
                                    GradientStop::new(0.0, Color::BLACK),
                                    GradientStop::new(1.0, Color::BLACK),
                                ],
                                SpreadMode::Pad,
                                Transform::identity(),
                            )
                            .unwrap()
                        });
                        paint.shader = gradient;
                    } else {
                        paint.set_color(crate::color::convert_color(
                            &proc.current_state.fill_color,
                            proc.current_state.fill_alpha,
                        ));
                    }

                    if let Some(mask) = current_clip.as_ref() {
                        let mut temp_pixmap = Pixmap::new(width, height).unwrap();
                        temp_pixmap.fill_path(&path, &paint, FillRule::Winding, final_transform, None);
                        temp_pixmap.apply_mask(mask);
                        pixmap.draw_pixmap(0, 0, temp_pixmap.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
                    } else {
                        pixmap.fill_path(&path, &paint, FillRule::Winding, final_transform, None);
                    }
                }
                path_builder = PathBuilder::new();
            }
            "sh" => {
                // Shading pattern. It fills the current clipping region.
                let mut paint = Paint::default();

                if let ColorSpace::LinearGradient {
                    start,
                    end,
                    start_color,
                    end_color,
                } = proc.current_state.fill_color
                {
                    let c1 = Color::from_rgba(
                        start_color.0,
                        start_color.1,
                        start_color.2,
                        proc.current_state.fill_alpha,
                    )
                    .unwrap_or(Color::BLACK);
                    let c2 = Color::from_rgba(
                        end_color.0,
                        end_color.1,
                        end_color.2,
                        proc.current_state.fill_alpha,
                    )
                    .unwrap_or(Color::BLACK);

                    if let Some(gradient) = LinearGradient::new(
                        Point::from_xy(start.0, start.1),
                        Point::from_xy(end.0, end.1),
                        vec![GradientStop::new(0.0, c1), GradientStop::new(1.0, c2)],
                        SpreadMode::Pad,
                        Transform::identity(),
                    ) {
                        paint.shader = gradient;
                    }
                } else {
                    paint.set_color(crate::color::convert_color(
                        &proc.current_state.fill_color,
                        proc.current_state.fill_alpha,
                    ));
                }

                let bounds_path = PathBuilder::from_rect(
                    tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32).unwrap(),
                );

                if let Some(mask) = current_clip.as_ref() {
                    let mut temp_pixmap = Pixmap::new(width, height).unwrap();
                    temp_pixmap.fill_path(&bounds_path, &paint, FillRule::Winding, Transform::identity(), None);
                    temp_pixmap.apply_mask(mask);
                    pixmap.draw_pixmap(0, 0, temp_pixmap.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
                } else {
                    pixmap.fill_path(&bounds_path, &paint, FillRule::Winding, Transform::identity(), None);
                }
            }
            _ => {
                // gs is parsed here if needed to set proc.current_state.fill_alpha
                // but we assume ExtGState parsing sets it inside GraphicsStateProcessor
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

#[cfg(test)]
mod tests_render_transparency {
    use super::*;
    use crate::content::ContentOperation;
    use crate::object::PdfObject;

    #[test]
    fn test_render_with_transparency() {
        let ops = vec![
            ContentOperation {
                operator: "ca".into(),
                operands: vec![PdfObject::Real(0.5)],
            },
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
            },
            ContentOperation {
                operator: "f".into(),
                operands: vec![],
            },
        ];

        let width = 200;
        let height = 200;
        let pixels = render_page_to_pixels(width, height, &ops).unwrap();

        let x = 100;
        let y = 100;
        let idx = ((y * width + x) * 4) as usize;

        // Since it's black with 0.5 alpha, the exact rgba mapped values might be subject to tiny-skia.
        // It converts premultiplied alpha or regular alpha to u8.
        // We know A should be around 127.
        assert!(pixels[idx + 3] > 0 && pixels[idx + 3] < 255);
    }
}
