# TASK-019: 2D Graphics Rasterization

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
Convert the PDF coordinate system and Graphics State (from TASK-010) into actual pixels by integrating a 2D rendering engine.

## Acceptance Criteria
- [ ] Evaluate and integrate `skia-safe` or `raqote`.
- [ ] Implement `render_page(page_id, width, height) -> Vec<u8>` returning an RGBA pixel buffer.
- [ ] Support basic path drawing (lines, rects, fill, stroke).
