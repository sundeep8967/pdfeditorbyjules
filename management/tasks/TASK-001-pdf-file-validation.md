# TASK-001: PDF File Validation and Basic I/O

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
The engine needs the ability to open a file path, read bytes, and verify that the file is actually a PDF by checking the header magic bytes (e.g., `%PDF-1.4`).

## Acceptance Criteria
- [ ] Create a `PdfDocument::open(path: &Path)` function.
- [ ] Function reads the first 1024 bytes and looks for the `%PDF-1.x` signature.
- [ ] Returns a robust `Result<PdfDocument, PdfError>` using the `thiserror` crate.
- [ ] Includes at least 2 unit tests (one valid file, one invalid file).

## Research Notes


## Blockers


## Linked RFC / ADR
ADR-001
