# TASK-007: FlateDecode and Object Streams

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Support PDF 1.5+ Object Streams (`ObjStm`) and general stream decompression using the `flate2` crate.

## Acceptance Criteria
- [ ] Implement a decompression utility using `flate2::read::ZlibDecoder`.
- [ ] When an object is in a `Compressed` XREF state, extract the parent `ObjStm`, decompress it, and extract the internal object.
- [ ] Integrate with the Parser (TASK-006) to yield a transparent `PdfObject`.
