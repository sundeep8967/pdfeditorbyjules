# TASK-003: Core PDF Object Model (AST)

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
Define the Rust `enum` that represents the primitive building blocks of a PDF file (Boolean, Integer, Real, String, Name, Array, Dictionary, Stream, Null, IndirectReference).

## Acceptance Criteria
- [ ] Create `PdfObject` enum with variants for all PDF data types.
- [ ] Create `PdfDictionary` type (likely wrapping a `HashMap<String, PdfObject>`).
- [ ] Ensure types implement `Debug` and `Clone` where appropriate.
- [ ] Memory footprint of `PdfObject` should be kept as small as possible.

## Research Notes


## Blockers


## Linked RFC / ADR
RFC-001
