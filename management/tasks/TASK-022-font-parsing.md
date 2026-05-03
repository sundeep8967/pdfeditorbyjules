# TASK-022: Full Font Subsystem

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Correctly map byte codes from text operators into Unicode strings using embedded font dictionaries, addressing the most complex part of the PDF specification.

## Acceptance Criteria
- [ ] Parse `/ToUnicode` CMap streams.
- [ ] Implement fallback to standard encodings (MacRoman, WinAnsi) if CMap is missing.
- [ ] Extract and pass raw embedded TrueType/Type1 font binaries to the renderer (TASK-019).
