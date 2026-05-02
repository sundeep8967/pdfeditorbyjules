# TASK-020: Map Graphics State to Canvas

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
The renderer currently uses hardcoded black colors and default stroke widths. It needs to read the `GraphicsState` from TASK-010 to apply dynamic stroke widths, line caps, line joins, and clipping paths to the 2D Canvas.

## Acceptance Criteria
- [ ] Apply `line_width` from `GraphicsState` to `tiny-skia` `Stroke` objects.
- [ ] Apply clipping paths using `Pixmap::set_clip_path`.
- [ ] Integrate with color spaces (TASK-021) to set fill and stroke paint colors.
