# TASK-027: Embedded TrueType Parser Integration

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Extract the raw `/FontFile2` byte stream from a PDF Font Descriptor, decompress it, and load it into a safe memory structure capable of resolving glyph mathematical outlines for rendering.

## Acceptance Criteria
- [ ] Add `ttf-parser` crate to `Cargo.toml`.
- [ ] Implement `load_truetype_font` in `src/font.rs` to initialize a `Face` object from the extracted bytes.
- [ ] Write unit tests to prove a valid TTF byte array loads without panicking.
