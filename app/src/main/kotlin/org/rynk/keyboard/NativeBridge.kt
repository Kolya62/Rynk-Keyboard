package org.rynk.keyboard

import android.graphics.Bitmap
import android.util.Log

object NativeBridge {
    private const val TAG = "RynkNativeBridge"
    private var isLoaded = false

    init {
        try {
            System.loadLibrary("rynk_core")
            isLoaded = true
            Log.i(TAG, "librynk_core.so successfully loaded into memory")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load librynk_core.so", e)
        }
    }

    fun isLibraryLoaded(): Boolean = isLoaded

    external fun nativeInit(width: Float, height: Float, density: Float, themeId: Int)
    external fun nativeDestroy()
    external fun nativeResize(width: Float, height: Float, density: Float)
    external fun nativeSetTheme(themeId: Int)
    external fun nativeSetLanguage(langId: Int)
    external fun nativeSetEnabledLanguages(langCodes: String)
    external fun nativeSetProfanityEnabled(enabled: Boolean)
    external fun nativeSetAutocorrectEnabled(enabled: Boolean)
    external fun nativeOnTouchEvent(action: Int, pointerId: Int, x: Float, y: Float, timeMs: Long): Boolean
    external fun nativeRender(bitmap: Bitmap, timeMs: Long): Boolean
    external fun nativePollEvents(): String
    external fun nativeGetTextLabels(): String
    external fun nativeReset()
    external fun nativeHandleBack(): Boolean
    external fun nativeIsBackspaceAt(x: Float, y: Float): Boolean
    external fun nativeRepeatBackspace(count: Int)
    external fun nativeGetLabelsVersion(): Long
    external fun nativeAddUserWord(word: String, isRu: Boolean)
    external fun nativeRemoveUserWord(word: String)
    external fun nativeGetUserWords(): String
    external fun nativeTick(timeMs: Long)
    external fun nativeSetClipboardText(text: String?)
    external fun nativeGetSuggestionAt(x: Float, y: Float): String
    external fun nativeGetMode(): Int
}

