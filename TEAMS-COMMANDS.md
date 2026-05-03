# LLM Agent Spawn Commands

**CEO Instructions:**
To scale development autonomously, spin up a new LLM agent instance and copy-paste the corresponding prompt below. Because of the architectural constraints defined in `AGENTS.md`, the agents will perfectly isolate their work.

---

### 1. Core Engine Team (Rust)
**Role:** Font & Typography Expert
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **Font & Typography Expert**. Read `management/engineering/rfc/research/TTF-PARSING.md`. Your objective is to expand `src/font.rs` to extract TrueType glyph geometries using `ttf-parser` and pass them to the rendering engine."

**Role:** Graphics & Rasterization Expert
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **Graphics Expert**. Read `PROGRESS.md` (Phase 6). Your objective is to expand `src/render.rs` to handle clipping paths, gradients, and transparency using `tiny-skia`."

**Role:** Security Expert
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **Security Expert**. Your objective is to expand `src/crypto.rs` by implementing the `aes` and `rc4` decryption logic to unlock encrypted PDF streams."

**Role:** CI/CD Build Engineer
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **CI/CD Build Engineer**. Your objective is to create `.github/workflows/build.yml`. Write the GitHub Actions workflow to cross-compile the Rust library into an iOS `.xcframework` using `cargo-lipo` and an Android `.aar` via the NDK."

---

### 2. Platform Bindings Team
**Role:** Lead Android Developer (Kotlin/JNI)
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Read `management/product/PRD-001-Commercial-SDK-V1.md`. Your objective is to create a new directory `sdk_bindings/android/`. Write the JNI C-bridge and the Kotlin `PdfDocument.kt` wrapper that exposes the `pdf_engine_core` C-ABI to Android developers."

**Role:** Lead iOS Developer (Swift)
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Read `management/product/PRD-001-Commercial-SDK-V1.md`. Your objective is to create a new directory `sdk_bindings/ios/`. Generate the C-Header (`pdf_engine.h`) and the Swift `PdfDocument.swift` wrapper that exposes the `pdf_engine_core` C-ABI to iOS developers."

**Role:** Web Developer (WASM)
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Your objective is to create `sdk_bindings/web/`. Configure `wasm-bindgen` to compile the core Rust engine into a WebAssembly module with TypeScript definitions."

**Role:** Cloud/Backend Integration Engineer
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Your objective is to create `sdk_bindings/node/`. Write the Node.js `ffi-napi` bindings to allow backend servers to use our Rust engine for headless PDF generation and editing."

---

### 3. Quality Assurance (QA) Team
**Role:** Security & Fuzz Tester
> "Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **QA Team**. Read `management/qa/TEST-PLAN-001-Core-Engine.md`. Create `tests/automation/fuzzer.py` to generate corrupted PDF files and feed them into the compiled Rust binary to test for memory leaks or crashes."

**Role:** Visual Regression Tester
> "Read `AGENTS.md`. You are assigned to the **QA Team**. Your objective is to create `tests/automation/visual_regression.py`. This script must render our test PDFs using the SDK, take the output RGBA buffer, and perform a pixel-diff comparison against Adobe Acrobat's output."

**Role:** Performance & Benchmarking Engineer
> "Read `AGENTS.md`. You are assigned to the **QA Team**. Your objective is to create `tests/automation/benchmark.rs` using the `criterion` crate. Ensure that our Rust `render_page_to_pixels` function consistently executes in under 50ms for complex vector pages."

---

### 4. Product & DX Team
**Role:** Technical Writer
> "Read `AGENTS.md`. You are assigned to the **Product & DX Team**. Read the C-ABI definitions in `pdf_engine_core/src/ffi.rs`. Your objective is to create a `docs/` directory and write enterprise-grade API documentation (Markdown/HTML) explaining how external customers should use our SDK."

---

### 5. Demo App Team
**Role:** iOS/Android UI Developer
> "Read `AGENTS.md`. You are assigned to the **Demo App Team**. Do not write PDF logic. Create a new repository `demo_apps/android/`. Build a beautiful, modern Android PDF Viewer app in Jetpack Compose that imports and uses the `.aar` file produced by the Platform Bindings team."

**Role:** Web UI Developer
> "Read `AGENTS.md`. You are assigned to the **Demo App Team**. Do not write PDF logic. Create `demo_apps/web/`. Build a beautiful React/Next.js PDF Editor interface that imports the WASM package from the Platform Bindings team."
