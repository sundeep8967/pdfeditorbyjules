# Research: Embedded TrueType Parsing in PDF

## Context
To reach Adobe-level rendering quality, our engine must parse embedded TrueType fonts (`/FontFile2` streams in PDF). Failing to do so causes text to render as missing glyphs or default system fonts.

## Specifications
According to the ISO 32000-1 PDF Specification (Sec 9.9) and the Apple/Microsoft TrueType spec:
1. TrueType fonts are embedded raw as Zlib compressed streams.
2. Once decompressed (via our `flate2` TASK-007), they are a contiguous binary block.
3. The font file begins with an Offset Table (12 bytes), followed by a Table Directory.

### Critical Tables for PDF Extraction
- `cmap` (Character to Glyph Index Mapping Table): We must parse Format 4 (segmented mapping) or Format 12 (UTF-32) to map PDF character codes to physical glyphs.
- `glyf` (Glyph Data): Contains the mathematical Bézier curves that represent the actual shape of the text.
- `loca` (Index to Location): Tells the engine where inside the `glyf` table a specific character's math is located.
- `hmtx` (Horizontal Metrics): Defines the width and kerning of the character, crucial for making sure text doesn't overlap on the screen.

## Architectural Decision
Parsing these byte-offsets manually is highly prone to buffer overflows and `panic!` states due to malicious or corrupted PDFs.
Instead of writing a million lines of parsing math from scratch, we will integrate the `ttf-parser` crate.
- It is `no_std`, zero-allocation, safe Rust.
- Licensed under Apache 2.0 / MIT (adhering to our strict ADR-002 dependency policy).
- It handles the complex math of traversing `cmap` and `glyf` safely.

## References
1. Adobe PDF Reference, Sixth Edition, v1.7.
2. Apple TrueType Reference Manual (https://developer.apple.com/fonts/TrueType-Reference-Manual/)
