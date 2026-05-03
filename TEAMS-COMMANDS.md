# LLM Agent Spawn Commands

**CEO Instructions:**
To scale development autonomously, spin up a new LLM agent instance and copy-paste **ONLY ONE** of the exact code block prompts below per agent. Do not copy an entire section, as giving an agent multiple roles will overload its context and cause confusion.

Because of the architectural constraints defined in `AGENTS.md`, the agents will perfectly isolate their work as long as they are given a single role.

---

### 1. Core Engine Team (Rust)

**Role:** Font & Typography Expert
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **Font & Typography Expert**. Read `management/engineering/rfc/research/TTF-PARSING.md`. Your objective is to expand `src/font.rs` to extract TrueType glyph geometries using `ttf-parser` and pass them to the rendering engine. Do not perform any other roles.
```

**Role:** Graphics & Rasterization Expert
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **Graphics Expert**. Read `PROGRESS.md` (Phase 6). Your objective is to expand `src/render.rs` to handle clipping paths, gradients, and transparency using `tiny-skia`. Do not perform any other roles.
```

**Role:** Security Expert
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **Security Expert**. Your objective is to expand `src/crypto.rs` by implementing the `aes` and `rc4` decryption logic to unlock encrypted PDF streams. Do not perform any other roles.
```

**Role:** CI/CD Build Engineer
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Core Engine Team**. Your role is the **CI/CD Build Engineer**. Your objective is to create `.github/workflows/build.yml`. Write the GitHub Actions workflow to cross-compile the Rust library into an iOS `.xcframework` using `cargo-lipo` and an Android `.aar` via the NDK. Do not perform any other roles.
```

---

### 2. Platform Bindings Team

**Role:** Lead Android Developer (Kotlin/JNI)
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Read `management/product/PRD-001-Commercial-SDK-V1.md`. Your objective is to create a new directory `sdk_bindings/android/`. Write the JNI C-bridge and the Kotlin `PdfDocument.kt` wrapper that exposes the `pdf_engine_core` C-ABI to Android developers. Do not perform any other roles.
```

**Role:** Lead iOS Developer (Swift)
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Read `management/product/PRD-001-Commercial-SDK-V1.md`. Your objective is to create a new directory `sdk_bindings/ios/`. Generate the C-Header (`pdf_engine.h`) and the Swift `PdfDocument.swift` wrapper that exposes the `pdf_engine_core` C-ABI to iOS developers. Do not perform any other roles.
```

**Role:** Web Developer (WASM)
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Your objective is to create `sdk_bindings/web/`. Configure `wasm-bindgen` to compile the core Rust engine into a WebAssembly module with TypeScript definitions. Do not perform any other roles.
```

**Role:** Cloud/Backend Integration Engineer
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **Platform Bindings Team**. Your objective is to create `sdk_bindings/node/`. Write the Node.js `ffi-napi` bindings to allow backend servers to use our Rust engine for headless PDF generation and editing. Do not perform any other roles.
```

---

### 3. Quality Assurance (QA) Team

**Role:** Security & Fuzz Tester
```text
Read `AGENTS.md` and `management/teams/TEAM-STRUCTURE.md`. You are assigned to the **QA Team**. Read `management/qa/TEST-PLAN-001-Core-Engine.md`. Create `tests/automation/fuzzer.py` to generate corrupted PDF files and feed them into the compiled Rust binary to test for memory leaks or crashes. Use ctypes to load the shared library. Do not perform any other roles.
```

**Role:** Visual Regression Tester
```text
Read `AGENTS.md`. You are assigned to the **QA Team**. Your objective is to create `tests/automation/visual_regression.py`. This script must render our test PDFs using the SDK, take the output RGBA buffer, and perform a pixel-diff comparison against Adobe Acrobat's output. Do not perform any other roles.
```

**Role:** Performance & Benchmarking Engineer
```text
Read `AGENTS.md`. You are assigned to the **QA Team**. Your objective is to create `tests/automation/benchmark.rs` using the `criterion` crate. Ensure that our Rust `render_page_to_pixels` function consistently executes in under 50ms for complex vector pages. Do not perform any other roles.
```

---

### 4. Product & DX Team

**Role:** Technical Writer
```text
Read `AGENTS.md`. You are assigned to the **Product & DX Team**. Read the C-ABI definitions in `pdf_engine_core/src/ffi.rs`. Your objective is to create a `docs/` directory and write enterprise-grade API documentation using mdBook explaining how external customers should use our SDK. Do not perform any other roles.
```

---

### 5. Demo App Team

**Role:** iOS/Android UI Developer
```text
Read `AGENTS.md`. You are assigned to the **Demo App Team**. Do not write PDF logic. Create a new directory `demo_apps/android/`. Build a beautiful, modern Android PDF Viewer app in Jetpack Compose. Mock the import of the `.aar` file and SDK layer for now. Do not perform any other roles.
```

**Role:** Web UI Developer
```text
Read `AGENTS.md`. You are assigned to the **Demo App Team**. Do not write PDF logic. Create `demo_apps/web/`. Build a beautiful React/Next.js PDF Editor interface. Mock the import of the WASM package and SDK layer for now. Do not perform any other roles.
```
