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

    // InputFieldMode constants
    const val INPUT_MODE_NORMAL = 0
    const val INPUT_MODE_PASSWORD = 1
    const val INPUT_MODE_VISIBLE_PASSWORD = 2
    const val INPUT_MODE_EMAIL = 3
    const val INPUT_MODE_URI = 4
    const val INPUT_MODE_NUMBER = 5
    const val INPUT_MODE_PHONE = 6
    const val INPUT_MODE_DATE = 7
    const val INPUT_MODE_TIME = 8
    const val INPUT_MODE_MULTILINE = 9
    const val INPUT_MODE_NUMBER_PASSWORD = 10
    const val INPUT_MODE_SENSITIVE = 11

    // Binary event constants
    const val EVENT_COMMIT_TEXT = 1
    const val EVENT_DELETE_SURROUNDING = 2
    const val EVENT_SEND_KEY_EVENT = 3
    const val EVENT_PERFORM_HAPTIC = 4
    const val EVENT_MOVE_CURSOR = 5
    const val EVENT_DELETE_WORD = 6
    const val EVENT_OPEN_SETTINGS = 7
    const val EVENT_SWITCH_IME = 8
    const val EVENT_HIDE_KEYBOARD = 9
    const val EVENT_CLEAR_CLIPBOARD = 10
    const val EVENT_CLIPBOARD_PASTED = 11

    external fun nativeInit(width: Float, height: Float, density: Float, themeId: Int)
    external fun nativeDestroy()
    external fun nativeResize(width: Float, height: Float, density: Float)
    external fun nativeSetTheme(themeId: Int)
    external fun nativeSetLanguage(langId: Int)
    external fun nativeSetEnabledLanguages(langCodes: String)
    external fun nativeSetProfanityEnabled(enabled: Boolean)
    external fun nativeSetAutocorrectEnabled(enabled: Boolean)
    external fun nativeSetMode(modeId: Int)
    external fun nativeOnTouchEvent(action: Int, pointerId: Int, x: Float, y: Float, timeMs: Long): Boolean
    external fun nativeRender(bitmap: Bitmap, timeMs: Long): Boolean
    external fun nativePollEvents(): String
    external fun nativePollEventsBinary(): ByteArray?
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

    // Production-ready enhancements: InputFieldMode, Popups & Adaptive Dictionary
    external fun nativeSetInputFieldMode(modeId: Int)
    external fun nativeSetPopupEnabled(enabled: Boolean)
    external fun nativeSetAdaptiveLearningEnabled(enabled: Boolean)
    external fun nativeClearAdaptiveData()
    external fun nativeSaveAdaptiveData(): ByteArray?
    external fun nativeLoadAdaptiveData(data: ByteArray)
}

