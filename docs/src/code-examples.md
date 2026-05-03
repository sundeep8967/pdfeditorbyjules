# Code Examples

This section provides practical examples of integrating with the PDF Editor SDK using C. These examples demonstrate standard patterns for invoking the SDK and strictly adhering to the [Memory Management](memory-management.md) rules.

## Full Lifecycle Example

This complete example demonstrates the core lifecycle:
1. Opening a document
2. Retrieving document metadata (version, page count)
3. Extracting text from the first page
4. Rendering the first page
5. Replacing text
6. Saving the document
7. Safely freeing all allocated resources

```c
#include <stdio.h>
#include <stdint.h>

// Forward declarations matching the C-ABI
typedef struct DocumentHandle DocumentHandle;

typedef struct {
    uint8_t *data;
    size_t size;
    uint32_t width;
    uint32_t height;
} PixelBuffer;

typedef struct {
    float x;
    float y;
    float width;
    float height;
    char *text_ptr;
} FFITextBoundingBox;

typedef struct {
    FFITextBoundingBox *boxes;
    size_t count;
} FFITextArray;

// External function declarations (imported from the Rust SDK)
extern DocumentHandle* pdf_engine_open_document(const char* path);
extern void pdf_engine_free_document(DocumentHandle* handle);
extern char* pdf_engine_get_version(DocumentHandle* handle);
extern void pdf_engine_free_string(char* s);
extern int32_t pdf_engine_get_page_count(DocumentHandle* handle);
extern FFITextArray pdf_engine_extract_page_text(DocumentHandle* handle, size_t page_index);
extern void pdf_engine_free_text_array(FFITextArray array);
extern PixelBuffer pdf_engine_render_page(DocumentHandle* handle, size_t page_index, uint32_t width, uint32_t height);
extern void pdf_engine_free_pixel_buffer(PixelBuffer buffer);
extern int32_t pdf_engine_replace_text(DocumentHandle* handle, size_t page_index, const char* target, const char* replacement);
extern int32_t pdf_engine_save_optimized(DocumentHandle* handle, const char* path);


int main() {
    const char* input_path = "sample.pdf";
    const char* output_path = "sample_edited.pdf";

    printf("Opening document: %s\n", input_path);
    DocumentHandle* doc = pdf_engine_open_document(input_path);
    if (!doc) {
        printf("Failed to open document.\n");
        return 1;
    }

    // 1. Get Metadata
    char* version = pdf_engine_get_version(doc);
    if (version) {
        printf("PDF Version: %s\n", version);
        pdf_engine_free_string(version); // FREE allocated string
    }

    int32_t page_count = pdf_engine_get_page_count(doc);
    printf("Total Pages: %d\n", page_count);

    if (page_count > 0) {
        // 2. Extract Text from Page 0
        printf("\nExtracting text from page 0...\n");
        FFITextArray text_array = pdf_engine_extract_page_text(doc, 0);

        for (size_t i = 0; i < text_array.count; i++) {
            FFITextBoundingBox box = text_array.boxes[i];
            printf("Found Text: '%s' at (X: %.2f, Y: %.2f)\n", box.text_ptr, box.x, box.y);
        }

        pdf_engine_free_text_array(text_array); // FREE text array and inner strings

        // 3. Render Page 0
        printf("\nRendering page 0 at 800x600...\n");
        PixelBuffer pixels = pdf_engine_render_page(doc, 0, 800, 600);

        if (pixels.data) {
            printf("Render successful. Buffer size: %zu bytes\n", pixels.size);
            // ... (Use pixel data here, e.g., display or save to image) ...
        } else {
            printf("Render failed.\n");
        }

        pdf_engine_free_pixel_buffer(pixels); // FREE pixel buffer

        // 4. Edit Text on Page 0
        printf("\nReplacing 'Hello' with 'World' on page 0...\n");
        int32_t replacements = pdf_engine_replace_text(doc, 0, "Hello", "World");
        if (replacements >= 0) {
             printf("Replaced %d instances.\n", replacements);
        } else {
             printf("Text replacement failed.\n");
        }

        // 5. Save Changes
        printf("\nSaving optimized document to: %s\n", output_path);
        int32_t save_status = pdf_engine_save_optimized(doc, output_path);
        if (save_status == 0) {
             printf("Save successful.\n");
        } else {
             printf("Save failed.\n");
        }
    }

    // 6. Cleanup Document
    printf("\nFreeing document handle...\n");
    pdf_engine_free_document(doc); // FREE main document handle

    printf("Done.\n");
    return 0;
}
```
