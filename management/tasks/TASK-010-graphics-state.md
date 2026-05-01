# TASK-010: Build Graphics State Machine

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [ ] Done

## Owner
@unassigned

## Objective
PDF content streams are executed sequentially. The engine must track the current graphics state (CTM, text matrix, font, color, line width) to know *where* and *how* text/graphics are rendered.

## Acceptance Criteria
- [ ] Implement a `GraphicsState` struct.
- [ ] Implement a `StateStack` to handle `q` (save) and `Q` (restore) operators.
- [ ] Track CTM (`cm`) and Text Matrix (`Tm`) transformations.
