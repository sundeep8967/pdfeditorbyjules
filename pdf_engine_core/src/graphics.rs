use crate::content::ContentOperation;
use crate::error::PdfError;
use crate::object::PdfObject;

/// A 3x3 transformation matrix used for coordinate and text transformations.
/// PDF matrices are defined by 6 numbers: [a b c d e f].
/// The matrix looks like:
/// | a  b  0 |
/// | c  d  0 |
/// | e  f  1 |
#[derive(Debug, Clone, PartialEq)]
pub struct TransformMatrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

impl TransformMatrix {
    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// Multiply this matrix by another: Self = Other * Self (PDF spec standard)
    pub fn multiply(&mut self, other: &TransformMatrix) {
        let new_a = other.a * self.a + other.b * self.c;
        let new_b = other.a * self.b + other.b * self.d;
        let new_c = other.c * self.a + other.d * self.c;
        let new_d = other.c * self.b + other.d * self.d;
        let new_e = other.e * self.a + other.f * self.c + self.e;
        let new_f = other.e * self.b + other.f * self.d + self.f;

        self.a = new_a;
        self.b = new_b;
        self.c = new_c;
        self.d = new_d;
        self.e = new_e;
        self.f = new_f;
    }
}

#[derive(Debug, Clone)]
pub struct GraphicsState {
    /// Current Transformation Matrix
    pub ctm: TransformMatrix,

    // Line state
    pub line_width: f32,
    pub fill_color: ColorSpace,
    pub stroke_color: ColorSpace,
    pub fill_alpha: f32,
    pub stroke_alpha: f32,

    // Text state
    pub character_spacing: f32,
    pub word_spacing: f32,
    pub horizontal_scaling: f32,
    pub leading: f32,
    pub font_name: Option<String>,
    pub font_size: f32,
    pub text_render_mode: i32,
    pub text_rise: f32,

    // Text object matrices (Not technically part of GraphicsState in spec, but easier to track here)
    pub text_matrix: TransformMatrix,
    pub text_line_matrix: TransformMatrix,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: TransformMatrix::default(),
            line_width: 1.0,
            fill_color: ColorSpace::default(),
            stroke_color: ColorSpace::default(),
            fill_alpha: 1.0,
            stroke_alpha: 1.0,
            character_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            leading: 0.0,
            font_name: None,
            font_size: 1.0,
            text_render_mode: 0,
            text_rise: 0.0,
            text_matrix: TransformMatrix::default(),
            text_line_matrix: TransformMatrix::default(),
        }
    }
}

/// The engine that processes operations and maintains state.
pub struct GraphicsStateProcessor {
    pub current_state: GraphicsState,
    state_stack: Vec<GraphicsState>,
}

impl Default for GraphicsStateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicsStateProcessor {
    pub fn new() -> Self {
        Self {
            current_state: GraphicsState::default(),
            state_stack: Vec::new(),
        }
    }

    /// Process a single content stream operation.
    pub fn process_op(&mut self, op: &ContentOperation) -> Result<(), PdfError> {
        match op.operator.as_str() {
            // -- Graphics State Save/Restore --
            "q" => {
                self.state_stack.push(self.current_state.clone());
            }
            "Q" => {
                if let Some(state) = self.state_stack.pop() {
                    self.current_state = state;
                } else {
                    return Err(PdfError::GraphicsStackUnderflow);
                }
            }
            // -- CTM --
            "cm" => {
                let m = extract_matrix_operands(&op.operands)?;
                self.current_state.ctm.multiply(&m);
            }
            // -- Color Spaces --
            "rg" => {
                self.current_state.fill_color = ColorSpace::RGB(
                    extract_f32(&op.operands, 0)?,
                    extract_f32(&op.operands, 1)?,
                    extract_f32(&op.operands, 2)?,
                );
            }
            "RG" => {
                self.current_state.stroke_color = ColorSpace::RGB(
                    extract_f32(&op.operands, 0)?,
                    extract_f32(&op.operands, 1)?,
                    extract_f32(&op.operands, 2)?,
                );
            }
            "k" => {
                self.current_state.fill_color = ColorSpace::CMYK(
                    extract_f32(&op.operands, 0)?,
                    extract_f32(&op.operands, 1)?,
                    extract_f32(&op.operands, 2)?,
                    extract_f32(&op.operands, 3)?,
                );
            }
            "K" => {
                self.current_state.stroke_color = ColorSpace::CMYK(
                    extract_f32(&op.operands, 0)?,
                    extract_f32(&op.operands, 1)?,
                    extract_f32(&op.operands, 2)?,
                    extract_f32(&op.operands, 3)?,
                );
            }
            "g" => {
                self.current_state.fill_color = ColorSpace::Gray(extract_f32(&op.operands, 0)?);
            }
            "G" => {
                self.current_state.stroke_color = ColorSpace::Gray(extract_f32(&op.operands, 0)?);
            }
            "w" => {
                self.current_state.line_width = extract_f32(&op.operands, 0)?;
            }
            // -- Text State --
            "Tc" => self.current_state.character_spacing = extract_f32(&op.operands, 0)?,
            "Tw" => self.current_state.word_spacing = extract_f32(&op.operands, 0)?,
            "Tz" => self.current_state.horizontal_scaling = extract_f32(&op.operands, 0)?,
            "TL" => self.current_state.leading = extract_f32(&op.operands, 0)?,
            "Tf" => {
                self.current_state.font_name = Some(extract_name(&op.operands, 0)?);
                self.current_state.font_size = extract_f32(&op.operands, 1)?;
            }
            "Tr" => self.current_state.text_render_mode = extract_f32(&op.operands, 0)? as i32,
            "Ts" => self.current_state.text_rise = extract_f32(&op.operands, 0)?,

            // -- Text Objects --
            "BT" => {
                self.current_state.text_matrix = TransformMatrix::default();
                self.current_state.text_line_matrix = TransformMatrix::default();
            }
            "ET" => {
                // Text object ends. State doesn't strictly reset, but good to know.
            }
            "Tm" => {
                let m = extract_matrix_operands(&op.operands)?;
                self.current_state.text_matrix = m.clone();
                self.current_state.text_line_matrix = m;
            }
            "Td" => {
                let tx = extract_f32(&op.operands, 0)?;
                let ty = extract_f32(&op.operands, 1)?;
                let m = TransformMatrix::new(1.0, 0.0, 0.0, 1.0, tx, ty);
                // Td sets Tlm = Tlm * offset, then Tm = Tlm
                self.current_state.text_line_matrix.multiply(&m);
                self.current_state.text_matrix = self.current_state.text_line_matrix.clone();
            }
            "TD" => {
                let tx = extract_f32(&op.operands, 0)?;
                let ty = extract_f32(&op.operands, 1)?;
                self.current_state.leading = -ty; // TD implicitly sets leading
                let m = TransformMatrix::new(1.0, 0.0, 0.0, 1.0, tx, ty);
                self.current_state.text_line_matrix.multiply(&m);
                self.current_state.text_matrix = self.current_state.text_line_matrix.clone();
            }
            "T*" => {
                let ty = -self.current_state.leading;
                let m = TransformMatrix::new(1.0, 0.0, 0.0, 1.0, 0.0, ty);
                self.current_state.text_line_matrix.multiply(&m);
                self.current_state.text_matrix = self.current_state.text_line_matrix.clone();
            }
            // Add other operators as necessary...
            "Do" => {
                let _name = extract_name(&op.operands, 0)?;
                // The PDF spec states that invoking an XObject acts as if it's wrapped in a q/Q pair.
                // In a fully flushed out rendering pipeline, we would look up `_name` in the Page's `/Resources /XObject` dictionary,
                // fetch the corresponding object ID from the Document, extract its stream, recursively parse it into ContentOperations,
                // and then invoke `self.process_op` on those inner operations.
                //
                // For MVP, we scaffold the isolation requirement to prove structural completeness.
                self.state_stack.push(self.current_state.clone());
                // recursive parsing goes here...
                if let Some(state) = self.state_stack.pop() {
                    self.current_state = state;
                }
            }
            _ => {} // Ignore unknown operators for now
        }
        Ok(())
    }
}

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

fn extract_name(operands: &[PdfObject], idx: usize) -> Result<String, PdfError> {
    if let Some(PdfObject::Name(n)) = operands.get(idx) {
        Ok(n.clone())
    } else {
        Err(PdfError::InvalidGraphicOperator("Expected Name".into()))
    }
}

fn extract_matrix_operands(operands: &[PdfObject]) -> Result<TransformMatrix, PdfError> {
    if operands.len() != 6 {
        return Err(PdfError::InvalidGraphicOperator(
            "Matrix requires 6 operands".into(),
        ));
    }
    Ok(TransformMatrix::new(
        extract_f32(operands, 0)?,
        extract_f32(operands, 1)?,
        extract_f32(operands, 2)?,
        extract_f32(operands, 3)?,
        extract_f32(operands, 4)?,
        extract_f32(operands, 5)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_restore_state() {
        let mut proc = GraphicsStateProcessor::new();

        proc.process_op(&ContentOperation {
            operator: "w".into(),                 // line width
            operands: vec![PdfObject::Real(5.0)], // Actually provide the operand
        })
        .unwrap();

        proc.process_op(&ContentOperation {
            operator: "q".into(),
            operands: vec![],
        })
        .unwrap();
        proc.current_state.line_width = 10.0;

        assert_eq!(proc.current_state.line_width, 10.0);

        proc.process_op(&ContentOperation {
            operator: "Q".into(),
            operands: vec![],
        })
        .unwrap();
        assert_eq!(proc.current_state.line_width, 5.0);
    }

    #[test]
    fn test_ctm_multiply() {
        let mut proc = GraphicsStateProcessor::new();
        // cm [2 0 0 2 10 10] -> scales by 2, translates by 10
        let op = ContentOperation {
            operator: "cm".into(),
            operands: vec![
                PdfObject::Integer(2),
                PdfObject::Integer(0),
                PdfObject::Integer(0),
                PdfObject::Integer(2),
                PdfObject::Integer(10),
                PdfObject::Integer(10),
            ],
        };

        proc.process_op(&op).unwrap();
        assert_eq!(proc.current_state.ctm.a, 2.0);
        assert_eq!(proc.current_state.ctm.e, 10.0);
    }
}

#[derive(Debug, PartialEq)]
pub struct ExtractedText {
    pub text: String,
    pub matrix: TransformMatrix,
    pub font_name: Option<String>,
}

impl GraphicsStateProcessor {
    /// Extracts text elements along with their calculated transformation matrices from a content stream.
    pub fn extract_text(
        &mut self,
        operations: &[ContentOperation],
    ) -> Result<Vec<ExtractedText>, PdfError> {
        let mut extracted = Vec::new();

        for op in operations {
            self.process_op(op)?;

            match op.operator.as_str() {
                "Tj" | "'" | "\"" => {
                    // Extract a simple string operand
                    if let Some(PdfObject::String(bytes)) = op.operands.first() {
                        // PDF strings might need CMap lookup.
                        // For MVP, we do a naive UTF-8 lossy conversion (which handles Standard ASCII/WinAnsi well enough).
                        let text_str = String::from_utf8_lossy(bytes).into_owned();

                        extracted.push(ExtractedText {
                            text: text_str,
                            matrix: self.current_state.text_matrix.clone(),
                            font_name: self.current_state.font_name.clone(),
                        });
                    }
                }
                "TJ" => {
                    // TJ takes an array of strings mixed with positioning numbers
                    if let Some(PdfObject::Array(arr)) = op.operands.first() {
                        let mut combined_text = String::new();
                        for item in arr {
                            if let PdfObject::String(bytes) = item {
                                combined_text.push_str(&String::from_utf8_lossy(bytes));
                            }
                            // We ignore the number offsets for base text extraction, they just adjust kerning
                        }

                        extracted.push(ExtractedText {
                            text: combined_text,
                            matrix: self.current_state.text_matrix.clone(),
                            font_name: self.current_state.font_name.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(extracted)
    }
}

#[cfg(test)]
mod text_tests {
    use super::*;

    #[test]
    fn test_extract_text_tj() {
        let mut proc = GraphicsStateProcessor::new();
        let ops = vec![
            ContentOperation {
                operator: "Tf".into(),
                operands: vec![PdfObject::Name("F1".into()), PdfObject::Integer(12)],
            },
            ContentOperation {
                operator: "Tj".into(),
                operands: vec![PdfObject::String(b"Hello".to_vec())],
            },
        ];

        let extracted = proc.extract_text(&ops).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].text, "Hello");
        assert_eq!(extracted[0].font_name, Some("F1".into()));
    }

    #[test]
    fn test_extract_text_tj_array() {
        let mut proc = GraphicsStateProcessor::new();
        let ops = vec![ContentOperation {
            operator: "TJ".into(),
            operands: vec![PdfObject::Array(vec![
                PdfObject::String(b"W".to_vec()),
                PdfObject::Integer(120),
                PdfObject::String(b"orld".to_vec()),
            ])],
        }];

        let extracted = proc.extract_text(&ops).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].text, "World"); // Ignoring the 120 kerning offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    RGB(f32, f32, f32),
    CMYK(f32, f32, f32, f32),
    Gray(f32),
    LinearGradient {
        start: (f32, f32),
        end: (f32, f32),
        start_color: (f32, f32, f32),
        end_color: (f32, f32, f32),
    },
}

impl Default for ColorSpace {
    fn default() -> Self {
        ColorSpace::Gray(0.0) // PDF default is black
    }
}

/// Represents the physical location of a piece of text on the rendered page.
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextBoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    // The actual text contained in this box
    pub text: String,
}

impl GraphicsStateProcessor {
    /// Calculates the physical bounding box for an extracted string of text.
    /// This requires multiplying the text width by the font size, horizontal scaling,
    /// character spacing, and finally projecting it through the Text Matrix (Tm) and Current Transformation Matrix (CTM).
    pub fn calculate_text_bounding_box(
        &self,
        text: &str,
        text_matrix: &TransformMatrix,
        font_size: f32,
        char_spacing: f32,
        word_spacing: f32,
        horizontal_scaling: f32,
        // In a full implementation, we pass the parsed `TrueTypeFont` here to get exact glyph widths.
        // For this baseline math, we assume a standard monospace width.
        _font_width_override: Option<f32>,
    ) -> TextBoundingBox {
        // PDF Spec 9.4.4: Text Space Details
        // The total displacement of a string is the sum of displacements of each character.
        // Displacement = (w0 - (Tj/1000)) * Tfs * Th + Tc + (Tw if space)

        let mut total_width = 0.0;
        let t_fs = font_size;
        let t_h = horizontal_scaling / 100.0;

        for c in text.chars() {
            // Standard PDF assumed glyph width in 1/1000ths of a unit if font is unknown
            let w0 = 600.0 / 1000.0;

            let mut char_width = w0 * t_fs * t_h + char_spacing;
            if c == ' ' {
                char_width += word_spacing;
            }
            total_width += char_width;
        }

        // The text box starts at the origin of the Text Matrix (Tm.e, Tm.f)
        // However, this must be projected through the CTM to get physical screen coordinates.
        let mut base_point = TransformMatrix::new(1.0, 0.0, 0.0, 1.0, text_matrix.e, text_matrix.f);
        // Self = CTM * BasePoint
        let ctm_clone = self.current_state.ctm.clone();
        base_point.multiply(&ctm_clone);

        // Height is roughly the font size projected through the CTM's vertical scaling
        let height = t_fs * ctm_clone.d;
        let physical_width = total_width * ctm_clone.a;

        TextBoundingBox {
            x: base_point.e,
            y: base_point.f,
            width: physical_width,
            height,
            text: text.to_string(),
        }
    }
}
