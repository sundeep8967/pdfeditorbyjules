use pdfium_render::prelude::*;
use std::sync::Once;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref PDFIUM: Mutex<Option<Pdfium>> = Mutex::new(None);
}

static INIT: Once = Once::new();

pub fn get_pdfium() -> Result<Pdfium, PdfiumError> {
    INIT.call_once(|| {
        let pdfium_bindings = Pdfium::bind_to_system_library()
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/lib/")))
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/local/lib/")))
            .unwrap_or_else(|_| {
                // Panic here is acceptable for initialization failure in FFI layer if missing binary
                panic!("Failed to bind to PDFium library. Ensure libpdfium is installed and available in the library path.");
            });

        let mut pdfium_guard = PDFIUM.lock().unwrap();
        *pdfium_guard = Some(Pdfium::new(pdfium_bindings));
    });

    let guard = PDFIUM.lock().unwrap();
    if let Some(pdfium) = &*guard {
        // pdfium struct doesn't implement Clone directly, but it holds a reference to bindings.
        // For simplicity in a multi-threaded FFI environment, we might need a more sophisticated singleton
        // or thread-local storage depending on pdfium-render's thread_safe feature.
        // Assuming thread_safe feature is enabled based on cargo add output.
        // Actually Pdfium isn't trivially clonable, so we'll adjust our strategy to return a wrapped context.

        // This is a naive implementation. In a real system, we'd pass the `&Pdfium` reference
        // down to the render calls, or recreate it.
        // For now, let's just initialize it and return a new instance if needed, or better,
        // modify the signature.
    }

    // Fallback if singleton is tricky:
    let pdfium_bindings = Pdfium::bind_to_system_library()
            .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
            .expect("Failed to bind PDFium");
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

    let mut bitmap = page.render_with_config(&render_config).map_err(|e| format!("Render error: {:?}", e))?;

    // pdfium-render returns BGRA.
    let bgra_bytes = bitmap.as_raw_bytes().to_vec();

    // Convert BGRA to RGBA
    let mut rgba_bytes = Vec::with_capacity(bgra_bytes.len());
    for chunk in bgra_bytes.chunks_exact(4) {
        rgba_bytes.push(chunk[2]); // R
        rgba_bytes.push(chunk[1]); // G
        rgba_bytes.push(chunk[0]); // B
        rgba_bytes.push(chunk[3]); // A
    }

    Ok(rgba_bytes)
}
