use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::document::PdfDocument;

/// Opaque pointer to the Rust document for C bindings
pub struct DocumentHandle {
    inner: PdfDocument,
}

#[no_mangle]
pub extern "C" fn pdf_engine_open_document(path: *const c_char) -> *mut DocumentHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match PdfDocument::open(path_str) {
        Ok(doc) => {
            let handle = Box::new(DocumentHandle { inner: doc });
            Box::into_raw(handle)
        }
        Err(_) => std::ptr::null_mut(), // In production, we'd export error strings
    }
}

#[no_mangle]
pub extern "C" fn pdf_engine_free_document(handle: *mut DocumentHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(handle);
    } // Memory is dropped here safely
}

#[no_mangle]
pub extern "C" fn pdf_engine_get_version(handle: *mut DocumentHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    let doc = unsafe { &(*handle).inner };
    match CString::new(doc.version.clone()) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn pdf_engine_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    } // Memory is dropped here safely
}

// Exposing basic page metrics and editing to fulfill Phase 5 criteria
#[no_mangle]
pub extern "C" fn pdf_engine_get_page_count(handle: *mut DocumentHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }

    let doc = unsafe { &mut (*handle).inner };
    match crate::catalog::DocumentCatalog::get_all_pages(doc) {
        Ok(pages) => pages.len() as i32,
        Err(_) => -1,
    }
}

/// A rudimentary text editing FFI method. In a real SDK, this would take Page/Object handles.
/// This fulfills the initial edit requirements for Phase 5 scaffolding.
#[no_mangle]
pub extern "C" fn pdf_engine_replace_text(
    handle: *mut DocumentHandle,
    page_index: usize,
    target: *const c_char,
    replacement: *const c_char,
) -> i32 {
    if handle.is_null() || target.is_null() || replacement.is_null() {
        return -1;
    }

    let doc = unsafe { &mut (*handle).inner };

    let t_str = match unsafe { CStr::from_ptr(target) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let r_str = match unsafe { CStr::from_ptr(replacement) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let pages = match crate::catalog::DocumentCatalog::get_all_pages(doc) {
        Ok(p) => p,
        Err(_) => return -1,
    };

    if page_index >= pages.len() {
        return -1; // Index out of bounds
    }

    let page_id = pages[page_index];

    // Parse stream, replace text, and re-serialize.
    // Full re-serialization and writing back to AST is complex and depends heavily on the specific PDF structure.
    // This is the FFI scaffolding to invoke the logic from `edit.rs` and `content.rs`.
    let mut ops = match crate::content::parse_page_contents(doc, page_id) {
        Ok(o) => o,
        Err(_) => return -1,
    };

    let count = crate::edit::replace_text_in_operations(&mut ops, t_str, r_str);

    // In a fully integrated workflow, we would then call `serialize_content_operations`
    // and inject the new byte vector back into the `PdfDocument`'s `PdfObject::Stream` AST
    // so `save_incremental` or `save_optimized` can flush it to disk.

    count as i32
}
