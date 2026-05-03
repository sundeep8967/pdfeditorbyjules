# TASK-021: Color Space Conversions

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@unassigned

## Objective
PDFs define colors in multiple spaces (RGB, CMYK, Grayscale). Our rendering engine (`tiny-skia`) only accepts RGBA. We must convert PDF color operators into `tiny-skia::Color`.

## Acceptance Criteria
- [ ] Implement `rg`/`RG` (DeviceRGB).
- [ ] Implement `k`/`K` (DeviceCMYK to RGB conversion).
- [ ] Implement `g`/`G` (DeviceGray).
