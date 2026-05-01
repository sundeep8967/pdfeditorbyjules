# TASK-002: XREF Table Discovery and Parser

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
To read any PDF, we must locate the Cross-Reference (XREF) table to know where every object lives in the file. The PDF spec requires reading the file from the *end* to find the `startxref` byte offset.

## Acceptance Criteria
- [ ] Logic to seek to the end of a file and read backwards to find the `startxref` keyword.
- [ ] Parse the byte offset following `startxref`.
- [ ] Jump to that offset and parse the XREF table (or XREF Stream in PDF 1.5+) into memory.
- [ ] Unit tests using dummy byte arrays mimicking PDF endings.

## Research Notes


## Blockers
Requires TASK-001

## Linked RFC / ADR
RFC-001
