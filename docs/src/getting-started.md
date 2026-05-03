# Getting Started

Welcome to the **PDF Editor SDK** documentation. This SDK provides a highly optimized, cross-platform PDF engine built entirely in Rust. This documentation focuses specifically on the **C-ABI (Application Binary Interface)** exposed by the core engine.

## Overview

The C-ABI serves as the lowest-level interface to the PDF engine. It allows developers to integrate the core PDF capabilities into any language that supports C interoperability (such as C/C++, Swift, Objective-C, Kotlin/JNI, or C#). The engine provides functionality for:
- Parsing and opening PDF documents.
- Reading document metadata.
- Extracting page text and layout bounding boxes.
- Replacing text content (editing).
- Rendering PDF pages to raw RGBA pixel buffers.
- Saving optimized output files.

## Integration Prerequisites

To link against the C-ABI, you will need the compiled artifacts from the core Rust engine. The artifacts will be built depending on your target platform:
- For Linux/Unix: `libpdf_engine_core.so`
- For macOS/iOS: `libpdf_engine_core.dylib` / `.a` / `.xcframework`
- For Windows: `pdf_engine_core.dll`
- For Android: Compiled `.so` files packaged within an `.aar`

### Basic Workflow

Interacting with the C-ABI follows a straightforward object-lifecycle pattern using opaque handles:
1. Initialize the engine by opening a document via `pdf_engine_open_document`, which returns a pointer to an opaque `DocumentHandle`.
2. Pass the `DocumentHandle` pointer to other API functions to perform tasks like rendering (`pdf_engine_render_page`) or text extraction (`pdf_engine_extract_page_text`).
3. Clean up the document handle via `pdf_engine_free_document`. **Failure to explicitly free memory allocated by the SDK will result in memory leaks.**

See the [Memory Management](memory-management.md) section for critical details.
