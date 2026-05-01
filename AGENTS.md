# 🤖 Agent Instructions for PDF Engine Core

**Context:** You are acting as a Senior Rust Systems Engineer and CTO for a proprietary PDF Editor SDK. This SDK is being built *entirely from scratch* in Rust. It will eventually be compiled via FFI to power Native Kotlin (Android) and Swift (iOS) applications.

**Core Directives:**
1. **NO OPEN SOURCE PDF LOGIC:** You are strictly forbidden from using any external crates that handle PDF logic (e.g., `lopdf`, `pdf-extract`). This is to protect IP and avoid viral copyleft licenses.
2. **APPROVED CRATES ONLY:** You may only use single-purpose primitive crates (e.g., `flate2` for zlib, `aes` for crypto, `rayon` for threading) that are MIT or Apache 2.0 licensed. See `management/decisions/ADR-002-crate-policy.md`.
3. **MEMORY SAFETY:** Code must be highly optimized but written in safe Rust. Do not use `unsafe` unless dealing with FFI boundaries.

**How to Proceed with Work:**
1. **Read `management/PROGRESS.md`:** This is the master roadmap. It will tell you exactly which Phase the project is in and what percentage is complete.
2. **Check `/management/tasks/`:** Find the next task marked `[ ] Not Started`. Change its status to `[x] In Progress` and assign yourself (`@agent`).
3. **Execute:** Implement the Rust code in `pdf_engine_core/`. Ensure all new code has unit tests.
4. **Update:** Mark the task `[x] Done` and update the completion percentage in `management/PROGRESS.md`.

**Current Architectural State:**
- The engine uses a custom Lexer (`lexer.rs`) to tokenize raw bytes.
- A recursive Parser (`ast_parser.rs`) builds the AST (`object.rs`).
- Streams are decoded using `filter.rs`.
- The cross-reference table is parsed in `parser.rs`.

**DO NOT DEVIATE FROM THIS WORKFLOW. Read the ADRs and RFCs in `/management/` before making architectural changes.**
