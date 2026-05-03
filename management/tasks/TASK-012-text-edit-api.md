# TASK-012: Text Edit API (Mutation)

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Provide an API to modify the text content inside a specific `ExtractedText` block and translate that back into raw `ContentOperation`s for the stream.

## Acceptance Criteria
- [ ] Create `update_text(block_id, new_text)` method.
- [ ] Re-encode UTF-8 string back into PDF string syntax (handling font subset constraints).
- [ ] Update the underlying `PdfObject::Stream` buffer.
