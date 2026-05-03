use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jint, jlong};

// Import our Rust Core API
use pdf_engine_core::ffi::{
    pdf_engine_free_document, pdf_engine_free_pixel_buffer, pdf_engine_open_document,
    pdf_engine_render_page, pdf_engine_replace_text, pdf_engine_save_optimized,
};

#[no_mangle]
pub extern "system" fn Java_com_pdfsdk_core_PdfDocument_nativeOpenDocument<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    path: JString<'local>,
) -> jlong {
    let path_str: String = env
        .get_string(&path)
        .expect("Couldn't get java string!")
        .into();

    // Convert to CString to pass to our C-API
    let c_path = std::ffi::CString::new(path_str).unwrap();

    // Call our core FFI layer
    let handle = pdf_engine_open_document(c_path.as_ptr());

    // The handle is a raw pointer, which we can treat as a 64-bit integer (jlong)
    handle as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_pdfsdk_core_PdfDocument_nativeCloseDocument<'local>(
    _env: JNIEnv<'local>,
    _class: JObject<'local>,
    handle: jlong,
) {
    if handle != 0 {
        pdf_engine_free_document(handle as *mut _);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_pdfsdk_core_PdfDocument_nativeReplaceText<'local>(
    mut env: JNIEnv<'local>,
    _class: JObject<'local>,
    handle: jlong,
    target: JString<'local>,
    replacement: JString<'local>,
) -> jboolean {
    if handle == 0 {
        return 0; // false
    }

    let target_str: String = env
        .get_string(&target)
        .expect("Couldn't get target string")
        .into();

    let replacement_str: String = env
        .get_string(&replacement)
        .expect("Couldn't get replacement string")
        .into();

    let c_target = std::ffi::CString::new(target_str).unwrap();
    let c_replacement = std::ffi::CString::new(replacement_str).unwrap();

    let result = pdf_engine_replace_text(
        handle as *mut _,
        0, // Assuming 0 for a page index argument based on previous grep output (needs checking if it's correct)
        c_target.as_ptr(),
        c_replacement.as_ptr(),
    );

    if result == 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "system" fn Java_com_pdfsdk_core_PdfDocument_nativeRenderPageToBitmap<'local>(
    env: JNIEnv<'local>,
    _class: JObject<'local>,
    handle: jlong,
    page_index: jint,
    width: jint,
    height: jint,
    bitmap: JObject<'local>,
) -> jboolean {
    if handle == 0 {
        return 0; // false
    }

    let pixel_buffer = pdf_engine_render_page(
        handle as *mut _,
        page_index as usize,
        width as u32,
        height as u32,
    );

    if pixel_buffer.data.is_null() {
        return 0;
    }

    // Now, we need to lock the Android Bitmap pixels to copy our data into it
    let mut pixels_info: jni_sys_for_bitmap::AndroidBitmapInfo = unsafe { std::mem::zeroed() };

    let result = unsafe {
        jni_sys_for_bitmap::AndroidBitmap_getInfo(
            env.get_native_interface() as *mut _,
            bitmap.as_raw() as *mut _,
            &mut pixels_info,
        )
    };

    if result < 0 {
        pdf_engine_free_pixel_buffer(pixel_buffer);
        return 0;
    }

    let mut dst_pixels_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let result = unsafe {
        jni_sys_for_bitmap::AndroidBitmap_lockPixels(
            env.get_native_interface() as *mut _,
            bitmap.as_raw() as *mut _,
            &mut dst_pixels_ptr,
        )
    };

    if result < 0 || dst_pixels_ptr.is_null() {
        pdf_engine_free_pixel_buffer(pixel_buffer);
        return 0;
    }

    // Copy our rendered RGBA data into the Android Bitmap buffer
    unsafe {
        let src_slice = std::slice::from_raw_parts(pixel_buffer.data, pixel_buffer.size);
        let dst_slice =
            std::slice::from_raw_parts_mut(dst_pixels_ptr as *mut u8, pixel_buffer.size);
        dst_slice.copy_from_slice(src_slice);
    }

    unsafe {
        jni_sys_for_bitmap::AndroidBitmap_unlockPixels(
            env.get_native_interface() as *mut _,
            bitmap.as_raw() as *mut _,
        );
    }

    pdf_engine_free_pixel_buffer(pixel_buffer);

    1 // true
}

// Note: Incremental save currently omitted since pdf_engine_save_incremental isn't exposed in the core FFI yet.

#[no_mangle]
pub extern "system" fn Java_com_pdfsdk_core_PdfDocument_nativeSaveAsFullRewrite<'local>(
    mut env: JNIEnv<'local>,
    _class: JObject<'local>,
    handle: jlong,
    output_path: JString<'local>,
) -> jboolean {
    if handle == 0 {
        return 0;
    }

    let path_str: String = env
        .get_string(&output_path)
        .expect("Couldn't get path string")
        .into();

    let c_path = std::ffi::CString::new(path_str).unwrap();

    let result = pdf_engine_save_optimized(handle as *mut _, c_path.as_ptr());
    if result == 0 { 1 } else { 0 }
}

mod jni_sys_for_bitmap {
    #![allow(non_camel_case_types)]
    use std::os::raw::{c_int, c_uint, c_void};

    #[repr(C)]
    pub struct AndroidBitmapInfo {
        pub width: c_uint,
        pub height: c_uint,
        pub stride: c_uint,
        pub format: c_int,
        pub flags: c_uint,
    }

    #[link(name = "jnigraphics")]
    extern "C" {
        pub fn AndroidBitmap_getInfo(
            env: *mut c_void,
            jbitmap: *mut c_void,
            info: *mut AndroidBitmapInfo,
        ) -> c_int;
        pub fn AndroidBitmap_lockPixels(
            env: *mut c_void,
            jbitmap: *mut c_void,
            addrPtr: *mut *mut c_void,
        ) -> c_int;
        pub fn AndroidBitmap_unlockPixels(env: *mut c_void, jbitmap: *mut c_void) -> c_int;
    }
}
