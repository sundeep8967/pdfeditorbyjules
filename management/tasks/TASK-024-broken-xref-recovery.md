# TASK-024: Broken XREF Recovery Heuristics

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
Adobe and PDFium can open heavily corrupted PDFs. Our engine must gracefully fallback if the `startxref` marker is missing or points to garbage data.

## Acceptance Criteria
- [ ] If `parse_xref_table` fails, trigger a linear scan heuristic.
- [ ] Scan the entire file byte-by-byte looking for `obj` and `endobj` markers.
- [ ] Dynamically rebuild the `XrefTable` in memory based on discovered offsets.
