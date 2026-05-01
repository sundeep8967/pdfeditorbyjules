# TASK-009: Parse Page Content Streams

## Status
[ ] Not Started | [x] In Progress | [ ] In Review | [ ] Done

## Owner
@jules

## Objective
For a given `/Page` object, extract its `/Contents` stream(s), decompress them (using TASK-007), and parse the resulting byte stream into graphics operators.

## Acceptance Criteria
- [ ] Extract `/Contents` (which could be a single reference or an array of references).
- [ ] Decompress the stream.
- [ ] Implement a Content Stream Lexer/Parser to extract operators (e.g., `q`, `Q`, `cm`, `BT`, `ET`, `Tj`) and their operands.
