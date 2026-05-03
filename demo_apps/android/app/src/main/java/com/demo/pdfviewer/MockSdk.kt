package com.demo.pdfviewer

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import kotlinx.coroutines.delay

class MockSdk {
    private var isOpened = false
    private var isModified = false

    suspend fun open(filePath: String): Boolean {
        // Simulate SDK initialization and parsing
        delay(800)
        isOpened = true
        isModified = false
        return true
    }

    suspend fun renderPage(pageIndex: Int): Bitmap {
        if (!isOpened) throw IllegalStateException("PDF not opened")
        // Simulate rendering time
        delay(300)

        // Return a mock Bitmap representing a PDF page
        val width = 800
        val height = 1100
        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)

        val paint = Paint()
        paint.color = Color.WHITE
        canvas.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)

        paint.color = Color.LTGRAY
        paint.style = Paint.Style.STROKE
        paint.strokeWidth = 10f
        canvas.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)

        paint.style = Paint.Style.FILL
        paint.color = Color.DKGRAY
        paint.textSize = 60f
        paint.textAlign = Paint.Align.CENTER
        canvas.drawText("Mock PDF Page ${pageIndex + 1}", width / 2f, height / 2f, paint)

        if (isModified) {
            paint.color = Color.BLUE
            paint.textSize = 40f
            canvas.drawText("(Modified)", width / 2f, height / 2f + 80f, paint)
        }

        return bitmap
    }

    suspend fun replaceText(searchString: String, replacementString: String): Int {
        if (!isOpened) throw IllegalStateException("PDF not opened")
        // Simulate text replacement processing
        delay(500)
        isModified = true
        return 1
    }

    suspend fun save(): Boolean {
        if (!isOpened) throw IllegalStateException("PDF not opened")
        // Simulate saving
        delay(1000)
        isModified = false
        return true
    }

    fun getPageCount(): Int {
        return if (isOpened) 5 else 0
    }
}
