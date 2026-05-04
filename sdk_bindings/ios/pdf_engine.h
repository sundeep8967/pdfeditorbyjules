#ifndef PDF_ENGINE_H
#define PDF_ENGINE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle for the PDF document
typedef struct DocumentHandle DocumentHandle;

// Pixel buffer struct returned by the renderer
typedef struct {
    uint8_t* data;
    size_t size;
    uint32_t width;
    uint32_t height;
} PixelBuffer;

// Text bounding box struct
typedef struct {
    float x;
    float y;
    float width;
    float height;
    char* text_ptr;
} FFITextBoundingBox;

// Text array struct
typedef struct {
    FFITextBoundingBox* boxes;
    size_t count;
} FFITextArray;

// --- Core FFI Methods ---

// Opens a PDF document from a file path
DocumentHandle* pdf_engine_open_document(const char* path);

// Frees the memory associated with a document handle
void pdf_engine_free_document(DocumentHandle* handle);

// Gets the version of the PDF engine
char* pdf_engine_get_version(DocumentHandle* handle);

// Frees strings returned by the engine
void pdf_engine_free_string(char* s);

// Gets the total number of pages in the document
int32_t pdf_engine_get_page_count(DocumentHandle* handle);

// Replaces text in the document
int32_t pdf_engine_replace_text(DocumentHandle* handle, size_t page_index, const char* target, const char* replacement);

// Renders a specific page to an RGBA pixel buffer
PixelBuffer pdf_engine_render_page(DocumentHandle* handle, size_t page_index, uint32_t width, uint32_t height);

// Frees the memory associated with a pixel buffer
void pdf_engine_free_pixel_buffer(PixelBuffer buffer);

// Saves the document using full rewrite (garbage collection)
int32_t pdf_engine_save_optimized(DocumentHandle* handle, const char* path);

// Extracts text bounding boxes for a given page
FFITextArray pdf_engine_extract_page_text(DocumentHandle* handle, size_t page_index);

// Frees the memory associated with an extracted text array
void pdf_engine_free_text_array(FFITextArray array);

#ifdef __cplusplus
}
#endif

#endif // PDF_ENGINE_H
