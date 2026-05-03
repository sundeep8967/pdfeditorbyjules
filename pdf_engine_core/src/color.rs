use crate::graphics::ColorSpace;
use tiny_skia::Color;

/// Converts a PDF ColorSpace into a tiny-skia Color (RGBA).
pub fn convert_color(cs: &ColorSpace) -> Color {
    match cs {
        ColorSpace::RGB(r, g, b) => Color::from_rgba(*r, *g, *b, 1.0).unwrap_or(Color::BLACK),
        ColorSpace::Gray(g) => Color::from_rgba(*g, *g, *g, 1.0).unwrap_or(Color::BLACK),
        ColorSpace::CMYK(c, m, y, k) => {
            // Standard naive CMYK to RGB conversion.
            // In an Adobe-level engine, this would use ICC profiles.
            let r = 1.0 - c.min(1.0);
            let g = 1.0 - m.min(1.0);
            let b = 1.0 - y.min(1.0);
            let k_inv = 1.0 - k.min(1.0);

            let final_r = r * k_inv;
            let final_g = g * k_inv;
            let final_b = b * k_inv;

            Color::from_rgba(final_r, final_g, final_b, 1.0).unwrap_or(Color::BLACK)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmyk_conversion() {
        let cmyk = ColorSpace::CMYK(0.0, 1.0, 0.0, 0.0); // Pure Magenta
        let rgb = convert_color(&cmyk);
        assert_eq!(rgb.red(), 1.0);
        assert_eq!(rgb.green(), 0.0);
        assert_eq!(rgb.blue(), 1.0);
    }
}
