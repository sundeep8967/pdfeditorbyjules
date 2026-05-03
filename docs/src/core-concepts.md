# Core Concepts

When using the C-ABI for the PDF Editor SDK, it is important to understand the fundamental concepts and data structures returned across the FFI (Foreign Function Interface) boundary.

## Opaque Handles

To maintain a stable C-ABI while leveraging Rust's powerful type system internally, the SDK uses **Opaque Handles**.

* **`DocumentHandle`**: The central object representing a loaded PDF document. The internal structure (`PdfDocument`) is hidden from C. You must treat `*mut DocumentHandle` as an opaque pointer. It is strictly your responsibility to pass this pointer back to the SDK to free it when you are done.

## Returned Structures (Structs)

Some API functions return raw C structures containing primitive data and pointers. You must understand how to handle these structures:

### `PixelBuffer`
Returned when rendering a page.
```c
typedef struct {
    uint8_t *data; // Pointer to raw RGBA pixel data
    size_t size;   // Total size of the buffer in bytes
    uint32_t width;
    uint32_t height;
} PixelBuffer;
```
This structure holds a pointer to pixel data allocated on the Rust heap. The layout is contiguous RGBA (`[R, G, B, A, R, G, B, A, ...]`).

### `FFITextBoundingBox` and `FFITextArray`
Returned when extracting text from a page for selection overlays or search logic.
```c
typedef struct {
    float x;
    float y;
    float width;
    float height;
    char *text_ptr; // Null-terminated C string
} FFITextBoundingBox;

typedef struct {
    FFITextBoundingBox *boxes; // Pointer to an array of boxes
    size_t count;              // Number of boxes in the array
} FFITextArray;
```
Each box provides spatial coordinates. The coordinate space uses typical PDF logic, or a simplified transformation defined by the internal CTM (Current Transformation Matrix).

## Editing Text

The basic text replacement API, `pdf_engine_replace_text`, performs a rudimentary find-and-replace operation on the PDF's content stream operations. It takes the target string and the replacement string as null-terminated C strings. Note that full re-serialization capabilities might vary based on the engine's current phase of development.

## Engine Save Mechanisms

The `pdf_engine_save_optimized` function attempts to write the modified AST back to disk. This process might utilize an Incremental Save architecture (to preserve signatures) or a Full Rewrite Save (garbage collection). The API call remains the same regardless of the internal implementation.
