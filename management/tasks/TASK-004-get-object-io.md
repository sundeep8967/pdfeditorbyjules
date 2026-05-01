# TASK-004: Raw Object Byte Extraction

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
Implement a mechanism on `PdfDocument` that, given an `ObjectId`, uses the `XrefTable` to seek to the correct byte offset and extract the raw bytes belonging to that object.

## Acceptance Criteria
- [ ] Implement `PdfDocument::get_raw_object_bytes(id: ObjectId) -> Result<Vec<u8>, PdfError>`.
- [ ] Must handle `InUse` objects by seeking to `byte_offset`.
- [ ] Must return an error or a Null placeholder for `Free` objects.
- [ ] Must return a specific error indicating "Requires Decompression" for `Compressed` objects (handled in TASK-007).
