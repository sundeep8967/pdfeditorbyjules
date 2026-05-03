# Engineering Organization Structure

This document defines the autonomous teams working on the PDF SDK.
**For LLM Agents:** You will be assigned to ONE of these teams. You must strictly adhere to your team's boundaries and NEVER modify code belonging to another team.

## 1. Core Engine Team (Rust & Infrastructure)
- **Directory Scope:** `pdf_engine_core/` and `.github/workflows/`
- **Role:** Deep PDF specification parsing, rendering math, cryptography, memory management, and cross-platform compilation.
- **Output:** A compiled C-ABI dynamic/static library (`.so`, `.a`, `.dylib`) and `ffi.rs`.
- **Constraint:** NEVER write Kotlin, Swift, or UI code.
- **Specific Roles:**
  - Font & Typography Expert
  - Graphics & Rasterization Expert
  - Security & Cryptography Expert
  - Memory & Performance Optimizer
  - **CI/CD Build Engineer** (Responsible for cross-compiling Rust to iOS `.xcframework` and Android NDK).

## 2. Platform Bindings Team (Mobile/Web/Cloud wrappers)
- **Directory Scope:** `sdk_bindings/`
- **Role:** Consume the C-ABI from the Core Engine Team. Write idiomatic wrappers in Kotlin (Android), Swift (iOS), TypeScript (WASM), and Node/Python (Cloud).
- **Constraint:** NEVER write Rust code. If an API is missing, request it from the Core Engine team via RFC.
- **Specific Roles:**
  - Lead Android Developer (Kotlin/JNI)
  - Lead iOS Developer (Swift)
  - Web Developer (WASM/TypeScript)
  - **Cloud/Backend Integration Engineer** (Node.js/Python FFI wrappers)

## 3. Quality Assurance (QA) & Performance Team
- **Directory Scope:** `management/qa/` and `tests/automation/`
- **Role:** Write Python/Bash automated fuzzing scripts, visual regression tests, memory leak detectors, and speed benchmarks.
- **Constraint:** Do not write feature code. Only write tests and break things.
- **Specific Roles:**
  - Security & Fuzz Tester
  - Visual Regression Tester
  - **Performance & Benchmarking Engineer** (Ensures <50ms page renders)

## 4. Product & Developer Experience (DX) Team
- **Directory Scope:** `management/product/` and `docs/`
- **Role:** Define product requirements and ensure external developers can actually use the SDK.
- **Constraint:** Do not write production feature code.
- **Specific Roles:**
  - Product Owner
  - Technical Writer (API Docs)

## 5. Demo App Team (Sales Engineering)
- **Directory Scope:** `demo_apps/`
- **Role:** Build Android, iOS, and Web consumer-facing apps that consume the SDK from the Platform Bindings team.
- **Constraint:** Treat the SDK as a black box. Build beautiful UI/UX.
- **Specific Roles:**
  - iOS UI Developer
  - Android UI Developer
  - **Web UI Developer** (React/Next.js)

## Cross-Team Protocol
All teams communicate via the `/management/tasks/` directory. If the Demo team needs a feature, the Product team writes a PRD, the Core team builds the Rust logic, the Bindings team wraps it, and the Demo team implements it in the UI.
