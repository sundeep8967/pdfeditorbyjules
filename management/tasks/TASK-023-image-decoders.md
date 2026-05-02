# TASK-023: Advanced Image Decoders

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Support decoding image streams embedded in the PDF so they can be rasterized by the rendering engine.

## Acceptance Criteria
- [ ] Implement `DCTDecode` (JPEG) using a safe Rust crate.
- [ ] Implement `JPXDecode` (JPEG2000).
- [ ] Implement `CCITTFaxDecode` and `JBIG2Decode`.
