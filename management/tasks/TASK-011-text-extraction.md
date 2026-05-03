# TASK-011: Base Text Extraction

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Using the Graphics State (TASK-010), map the raw byte codes in `Tj` or `TJ` operators to actual Unicode strings using the page's `/Resources` -> `/Font` dictionaries and CMap tables.

## Acceptance Criteria
- [ ] Load font dictionaries from page resources.
- [ ] Parse `ToUnicode` CMaps or standard encodings (WinAnsi/MacRoman).
- [ ] Convert raw text operator operands into UTF-8 strings.
