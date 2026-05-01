# ADR 001: Core Language Choice

## Status
Accepted

## Context
We are building a proprietary PDF Engine SDK from scratch capable of rendering and editing PDFs. It needs to run on Android, iOS, and eventually Web. We need absolute control, extreme performance, and memory safety to avoid the massive class of CVEs (buffer overflows, use-after-free) that plague C++ PDF engines.

## Decision
We will write the core engine strictly in **Rust**.
Rust provides the exact systems-level performance of C++ but guarantees memory safety at compile time. It also has excellent cross-compilation support via `cargo` for iOS (aarch64-apple-ios) and Android (aarch64-linux-android), and WebAssembly (wasm32-unknown-unknown).

## Consequences
- Requires designing a robust C-ABI FFI layer to communicate with Swift and Kotlin.
- Slightly slower initial development speed compared to dynamic languages due to the borrow checker, but massively reduced debugging and QA time later.
