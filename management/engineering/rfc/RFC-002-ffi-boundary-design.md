# RFC 002: FFI Boundary Design for iOS and Android

## Background
To use our Rust SDK in mobile apps, we need a stable interface. Swift and Kotlin cannot call Rust directly; they must call C-compatible functions.

## Proposal
We will expose a strictly C-compatible API using `#[no_mangle] pub extern "C"` functions.
- We will not expose the complex PDF object graph across the boundary. That is too complex and dangerous.
- We will expose a "Handle" based API.
  - `DocumentHandle open_document(const char* path)`
  - `int get_page_count(DocumentHandle doc)`
  - `void render_page(DocumentHandle doc, int page_num, uint8_t* buffer, int width, int height)`
- Memory allocated by Rust must be freed by Rust. We will provide `free_document(DocumentHandle doc)`.

## Tools
We will evaluate using `cbindgen` or `uniffi` to auto-generate the C headers, Swift bindings, and Kotlin JNA/JNI bindings.
