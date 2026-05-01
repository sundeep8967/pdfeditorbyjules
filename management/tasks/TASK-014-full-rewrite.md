# TASK-014: Full Rewrite Save (Garbage Collection)

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
Provide a "Save As" / "Optimize" feature that walks the entire object graph, skips `Free` or orphaned objects, and writes a completely fresh PDF file from scratch with a unified XREF table.

## Acceptance Criteria
- [ ] Traverse the object graph starting from `trailer` `/Root`.
- [ ] Serialize all reachable `PdfObject` AST elements into raw bytes.
- [ ] Generate a fresh XREF table from the newly written offsets.
