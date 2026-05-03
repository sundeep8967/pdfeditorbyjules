use pdfium_render::prelude::*;

// For a safe FFI boundary, we instantiate PDFium per-request without panicking.
// In a highly optimized version, the `Pdfium` instance would be stored inside `DocumentHandle`.
pub fn get_pdfium() -> Result<Pdfium, String> {
    let pdfium_bindings = Pdfium::bind_to_system_library()
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/lib/")))
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/local/lib/")))
            .map_err(|e| format!("Failed to dynamically bind PDFium: {:?}", e))?;

    Ok(Pdfium::new(pdfium_bindings))
}

pub fn render_page_with_pdfium(path: &str, page_index: usize, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let pdfium = get_pdfium().map_err(|e| format!("PDFium init error: {:?}", e))?;

    let document = pdfium.load_pdf_from_file(path, None).map_err(|e| format!("Load error: {:?}", e))?;
    let pages = document.pages();

    if page_index >= pages.len() as usize {
        return Err("Page index out of bounds".to_string());
    }

    let page = pages.get(page_index as i32).map_err(|e| format!("Page get error: {:?}", e))?;

    let render_config = PdfRenderConfig::new()
        .set_target_width(width as i32)
        .set_target_height(height as i32)
        .set_clear_color(PdfColor::WHITE);

    let bitmap = page.render_with_config(&render_config).map_err(|e| format!("Render error: {:?}", e))?;

    // Convert BGRA to RGBA
    let bgra_bytes = bitmap.as_raw_bytes().to_vec();
    let mut rgba_bytes = Vec::with_capacity(bgra_bytes.len());
    for chunk in bgra_bytes.chunks_exact(4) {
        rgba_bytes.push(chunk[2]); // R
        rgba_bytes.push(chunk[1]); // G
        rgba_bytes.push(chunk[0]); // B
        rgba_bytes.push(chunk[3]); // A
    }

    Ok(rgba_bytes)
}
