# Master Test Plan: Core Rust Engine

## Objective
Verify the absolute memory safety, specification compliance, and performance of the compiled Rust C-ABI engine.

## 1. Automated Fuzzing
- **Vector:** Feed 10,000 highly corrupted, randomly generated PDF files into `pdf_engine_open_document`.
- **Pass Condition:** The engine returns a null pointer or defined error code. It MUST NOT trigger a segmentation fault or `panic!`.

## 2. Memory Leak Verification
- **Vector:** Run the engine through `Valgrind` / `AddressSanitizer`.
- **Action:** Open document -> Render 50 pages -> Free pages -> Free document.
- **Pass Condition:** 0 bytes definitely lost. The FFI boundary must successfully reclaim all `Box::into_raw` allocations.

## 3. Visual Regression
- **Vector:** Render the standardized "PDF 32000-1 Compliance Test Suite" using `tiny-skia`.
- **Action:** Compare generated RGBA pixel buffers against golden master images from Adobe Acrobat using a pixel-diff algorithm.
- **Pass Condition:** 99.9% pixel match tolerance.

## 4. Cryptographic Verification
- **Vector:** Open PDFs encrypted with 40-bit RC4, 128-bit AES, and 256-bit AES.
- **Pass Condition:** Engine accurately derives the User/Owner keys and successfully decrypts text streams matching golden cleartext files.
