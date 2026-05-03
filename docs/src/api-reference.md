# API Reference

This section provides a detailed reference for all functions exported by the core PDF Engine via the C-ABI.

## Document Lifecycle

### `pdf_engine_open_document`
Opens a PDF document from a given file path.
```c
DocumentHandle* pdf_engine_open_document(const char* path);
```
* **Parameters:**
  * `path`: A null-terminated string representing the file path to the PDF.
* **Returns:** A pointer to an opaque `DocumentHandle`. Returns `NULL` if the path is invalid or the document cannot be opened.
* **Memory Ownership:** The caller takes ownership of the returned handle and must free it using `pdf_engine_free_document`.

### `pdf_engine_free_document`
Frees a previously opened document.
```c
void pdf_engine_free_document(DocumentHandle* handle);
```
* **Parameters:**
  * `handle`: The pointer returned by `pdf_engine_open_document`.
* **Behavior:** Safely does nothing if `handle` is `NULL`.

### `pdf_engine_save_optimized`
Saves the currently open document to a new file path.
```c
int32_t pdf_engine_save_optimized(DocumentHandle* handle, const char* path);
```
* **Parameters:**
  * `handle`: The open document handle.
  * `path`: A null-terminated string representing the destination file path.
* **Returns:** `0` on success, `-1` on failure (e.g., invalid handle, path error, or save failure).

## Document Information

### `pdf_engine_get_version`
Retrieves the PDF version string of the loaded document.
```c
char* pdf_engine_get_version(DocumentHandle* handle);
```
* **Parameters:**
  * `handle`: The open document handle.
* **Returns:** A null-terminated C string containing the version (e.g., "1.4"). Returns `NULL` on error.
* **Memory Ownership:** The caller takes ownership of the string and must free it using `pdf_engine_free_string`.

### `pdf_engine_get_page_count`
Retrieves the total number of pages in the document.
```c
int32_t pdf_engine_get_page_count(DocumentHandle* handle);
```
* **Parameters:**
  * `handle`: The open document handle.
* **Returns:** The number of pages, or `-1` if the handle is invalid or an error occurs.

## Rendering

### `pdf_engine_render_page`
Renders a specific page to an RGBA pixel buffer.
```c
PixelBuffer pdf_engine_render_page(
    DocumentHandle* handle,
    size_t page_index,
    uint32_t width,
    uint32_t height
);
```
* **Parameters:**
  * `handle`: The open document handle.
  * `page_index`: The zero-based index of the page to render.
  * `width`: The desired target width in pixels.
  * `height`: The desired target height in pixels.
* **Returns:** A `PixelBuffer` structure. If an error occurs, the `data` pointer inside the struct will be `NULL` and `size` will be `0`.
* **Memory Ownership:** The caller takes ownership of the underlying pixel data and must free it using `pdf_engine_free_pixel_buffer`.

### `pdf_engine_free_pixel_buffer`
Frees a pixel buffer.
```c
void pdf_engine_free_pixel_buffer(PixelBuffer buffer);
```
* **Parameters:**
  * `buffer`: The `PixelBuffer` returned by `pdf_engine_render_page`.
* **Behavior:** Safely does nothing if the `data` pointer is `NULL`.

## Text & Editing

### `pdf_engine_extract_page_text`
Extracts text and bounding box locations from a page.
```c
FFITextArray pdf_engine_extract_page_text(DocumentHandle* handle, size_t page_index);
```
* **Parameters:**
  * `handle`: The open document handle.
  * `page_index`: The zero-based index of the page.
* **Returns:** An `FFITextArray` structure containing layout boxes and text. On error, the `boxes` pointer will be `NULL` and `count` will be `0`.
* **Memory Ownership:** The caller takes ownership of the array and the inner strings, and must free the entire structure using `pdf_engine_free_text_array`.

### `pdf_engine_free_text_array`
Frees a text array.
```c
void pdf_engine_free_text_array(FFITextArray array);
```
* **Parameters:**
  * `array`: The `FFITextArray` returned by `pdf_engine_extract_page_text`.
* **Behavior:** Iterates through the array to safely free individual text strings, then frees the array itself. Safely does nothing if the `boxes` pointer is `NULL`.

### `pdf_engine_replace_text`
Replaces instances of target text with a replacement string on a specific page.
```c
int32_t pdf_engine_replace_text(
    DocumentHandle* handle,
    size_t page_index,
    const char* target,
    const char* replacement
);
```
* **Parameters:**
  * `handle`: The open document handle.
  * `page_index`: The zero-based index of the page.
  * `target`: A null-terminated C string containing the text to find.
  * `replacement`: A null-terminated C string containing the text to insert.
* **Returns:** The number of replacements made. Returns `-1` on error (e.g., out of bounds or invalid strings).

## Utility

### `pdf_engine_free_string`
Frees a string allocated by the SDK.
```c
void pdf_engine_free_string(char* s);
```
* **Parameters:**
  * `s`: A pointer to a C-string previously returned by an SDK function.
* **Behavior:** Safely does nothing if `s` is `NULL`.
