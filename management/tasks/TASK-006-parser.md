# TASK-006: PDF AST Parser

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Consume a stream of `PdfToken`s (from TASK-005) and assemble them into a valid `PdfObject` AST (from TASK-003).

## Acceptance Criteria
- [ ] Implement `Parser::parse_object() -> Result<PdfObject, PdfError>`.
- [ ] Handle recursive parsing (e.g., Arrays containing Dictionaries containing Strings).
- [ ] Handle indirect object definitions (`10 0 obj ... endobj`).
- [ ] Handle Stream parsing (Dictionary followed by `stream\r\n...\r\nendstream`).
