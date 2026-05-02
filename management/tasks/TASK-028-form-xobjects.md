# TASK-028: Form XObject Processing

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Support the `Do` operator, which invokes a reusable Form XObject (a self-contained content stream with its own resources and graphics state).

## Acceptance Criteria
- [ ] Add `do_xobject` processing in `graphics.rs`.
- [ ] Ensure that a Form XObject's execution isolates its graphics state (acts like it is wrapped in `q` and `Q`).
- [ ] Properly intercept `Do` tokens in the content stream.
