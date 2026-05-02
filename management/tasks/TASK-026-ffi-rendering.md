# TASK-026: FFI Rendering Hooks

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
The `tiny-skia` renderer creates RGBA buffers internally. We must expose an `extern "C"` method that allows a Swift/Kotlin app to request a specific page to be rendered into memory, and return a struct containing the pointer, width, and height.

## Acceptance Criteria
- [ ] Expose `pdf_engine_render_page`.
- [ ] Return a C-struct with `*mut u8`, `width`, and `height`.
- [ ] Expose `pdf_engine_free_pixel_buffer` to prevent memory leaks across the boundary.
