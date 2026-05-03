package com.pdfsdk.core

import android.graphics.Bitmap

/**
 * A wrapper class that interacts with the Rust PDF core engine via JNI.
 */
class PdfDocument private constructor(private var nativeHandle: Long) : AutoCloseable {

    companion object {
        init {
            System.loadLibrary("pdf_engine_core")
        }

        /**
         * Opens a PDF document from the given file path.
         * @param path The path to the PDF file.
         * @return A new PdfDocument instance, or null if the file could not be opened.
         */
        @JvmStatic
        fun open(path: String): PdfDocument? {
            val handle = nativeOpenDocument(path)
            return if (handle == 0L) {
                null
            } else {
                PdfDocument(handle)
            }
        }

        // --- Native JNI Methods ---
        @JvmStatic
        private external fun nativeOpenDocument(path: String): Long
    }

    /**
     * Replaces a specific string of text across the document.
     * @param target The exact text string to find.
     * @param replacement The new text to insert.
     * @return true if successful, false otherwise.
     */
    fun replaceText(target: String, replacement: String): Boolean {
        if (nativeHandle == 0L) return false
        return nativeReplaceText(nativeHandle, target, replacement)
    }

    /**
     * Renders a specific page to an Android Bitmap.
     * @param pageIndex 0-based page index.
     * @param width The desired width of the rendered bitmap.
     * @param height The desired height of the rendered bitmap.
     * @return A Bitmap object containing the rendered page, or null if rendering fails.
     */
    fun renderPage(pageIndex: Int, width: Int, height: Int): Bitmap? {
        if (nativeHandle == 0L) return null

        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val success = nativeRenderPageToBitmap(nativeHandle, pageIndex, width, height, bitmap)
        return if (success) {
            bitmap
        } else {
            bitmap.recycle()
            null
        }
    }

    /**
     * Saves the document incrementally to the same file.
     * Note: Not currently implemented in core FFI.
     * @return true if successful, false otherwise.
     */
    fun saveIncremental(): Boolean {
        if (nativeHandle == 0L) return false
        // return nativeSaveIncremental(nativeHandle)
        throw UnsupportedOperationException("Incremental save not yet supported by core engine.")
    }

    /**
     * Rewrites the document sequentially to a new file, discarding unused objects.
     * @param outputPath The path where the new PDF should be saved.
     * @return true if successful, false otherwise.
     */
    fun saveAsFullRewrite(outputPath: String): Boolean {
        if (nativeHandle == 0L) return false
        return nativeSaveAsFullRewrite(nativeHandle, outputPath)
    }

    override fun close() {
        if (nativeHandle != 0L) {
            nativeCloseDocument(nativeHandle)
            nativeHandle = 0L
        }
    }

    private external fun nativeReplaceText(handle: Long, target: String, replacement: String): Boolean
    private external fun nativeRenderPageToBitmap(handle: Long, pageIndex: Int, width: Int, height: Int, bitmap: Bitmap): Boolean
    // private external fun nativeSaveIncremental(handle: Long): Boolean
    private external fun nativeSaveAsFullRewrite(handle: Long, outputPath: String): Boolean
    private external fun nativeCloseDocument(handle: Long)
}
