# Memory Management

Proper memory management is the most critical aspect of integrating with the PDF Editor SDK via the C-ABI. The SDK is written in Rust, which uses its own heap allocator.

**Rule of Thumb: Any memory allocated by the SDK MUST be freed by the SDK.**

Failing to properly manage memory across the FFI boundary will result in memory leaks or, worse, undefined behavior (such as segmentation faults) if you attempt to use the wrong allocator (like the C `free()` function) to drop Rust-allocated memory.

## Ownership Transfer

Many functions in the C-ABI transfer ownership of heap-allocated memory from Rust to the caller. This means the pointer or structure returned by the function "belongs" to you. However, you are explicitly forbidden from freeing this memory yourself using `free()`. Instead, you must use the corresponding cleanup function provided by the SDK.

### Freeing the Document
When you open a document via `pdf_engine_open_document`, you receive a `*mut DocumentHandle`.
* **Must call:** `pdf_engine_free_document(handle)`
* **When:** As soon as you are completely finished with the document.

### Freeing Strings
Some API calls return a newly allocated C-string (e.g., `pdf_engine_get_version` returns a `*mut char`).
* **Must call:** `pdf_engine_free_string(ptr)`
* **When:** Once you have copied the string's contents into your own application's memory space or are otherwise done with it.

### Freeing Pixel Buffers
When you render a page via `pdf_engine_render_page`, you receive a `PixelBuffer` struct containing a pointer to the raw pixel data (`data`).
* **Must call:** `pdf_engine_free_pixel_buffer(buffer)`
* **When:** After you have uploaded the pixel data to a texture, copied it to an image object in your language, or displayed it. The entire struct is passed back by value.

### Freeing Text Arrays
When you extract text via `pdf_engine_extract_page_text`, you receive an `FFITextArray` struct. This struct holds a pointer to an array of boxes, and each box contains a pointer to a C-string (`text_ptr`).
* **Must call:** `pdf_engine_free_text_array(array)`
* **When:** Once you have processed the bounding boxes and text. The SDK safely iterates through the internal pointers and frees the strings as well as the outer array.

## Best Practices
1. **Never use `free()`:** Do not use `free()`, `delete`, or any other external deallocator on pointers returned by the SDK.
2. **Handle Null Pointers:** Always check for `NULL` pointers returned by functions. A `NULL` usually indicates an initialization failure, an error, or out-of-bounds access. The SDK's cleanup functions (`pdf_engine_free_*`) safely do nothing if passed a `NULL` pointer or empty struct.
3. **Copy Quickly:** For returned strings or short-lived data, copy the contents into your own host-language data structures (e.g., `std::string` in C++, `String` in Swift) immediately, and then invoke the SDK's free function.
