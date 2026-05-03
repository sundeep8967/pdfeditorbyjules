use pdf_engine_core::ffi::{
    pdf_engine_free_document, pdf_engine_free_pixel_buffer, pdf_engine_open_document,
    pdf_engine_render_page, pdf_engine_replace_text, pdf_engine_save_optimized, DocumentHandle, PixelBuffer,
};
use wasm_bindgen::prelude::*;
use std::ffi::CString;

#[wasm_bindgen]
pub struct WasmPdfDocument {
    handle: *mut DocumentHandle,
}

#[wasm_bindgen]
impl WasmPdfDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(path: &str) -> Option<WasmPdfDocument> {
        let c_path = CString::new(path).ok()?;
        let handle = pdf_engine_open_document(c_path.as_ptr());
        if handle.is_null() {
            None
        } else {
            Some(WasmPdfDocument { handle })
        }
    }

    pub fn replace_text(&self, page_index: usize, target: &str, replacement: &str) -> bool {
        if self.handle.is_null() {
            return false;
        }
        let c_target = CString::new(target).unwrap_or_default();
        let c_replacement = CString::new(replacement).unwrap_or_default();
        let result = pdf_engine_replace_text(
            self.handle,
            page_index,
            c_target.as_ptr(),
            c_replacement.as_ptr(),
        );
        result != 0
    }

    pub fn render_page(&self, page_index: usize, width: u32, height: u32) -> Option<js_sys::Uint8Array> {
        if self.handle.is_null() {
            return None;
        }

        let pixel_buffer = pdf_engine_render_page(self.handle, page_index, width, height);

        if pixel_buffer.data.is_null() {
            return None;
        }

        let slice = unsafe { std::slice::from_raw_parts(pixel_buffer.data, pixel_buffer.size) };
        let js_array = js_sys::Uint8Array::from(slice);

        // Free the buffer now that we've copied it to JS memory
        pdf_engine_free_pixel_buffer(pixel_buffer);

        Some(js_array)
    }

    pub fn save_as_full_rewrite(&self, output_path: &str) -> bool {
        if self.handle.is_null() {
            return false;
        }
        let c_path = CString::new(output_path).unwrap_or_default();
        let result = pdf_engine_save_optimized(self.handle, c_path.as_ptr());
        result == 0
    }
}

impl Drop for WasmPdfDocument {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            pdf_engine_free_document(self.handle);
            self.handle = std::ptr::null_mut();
        }
    }
}
