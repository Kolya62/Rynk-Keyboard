package org.rynk.keyboard

import android.app.AlertDialog
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.graphics.drawable.ColorDrawable
import android.inputmethodservice.InputMethodService
import android.os.Build
import android.view.KeyEvent
import android.view.View
import android.view.Window
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.widget.Toast

class RynkInputMethodService : InputMethodService() {

    private lateinit var hapticManager: HapticManager
    private var keyboardView: RynkKeyboardView? = null
    private var clipboardListener: ClipboardManager.OnPrimaryClipChangedListener? = null
    private var isDispatchingEvents = false

    override fun onCreate() {
        super.onCreate()
        hapticManager = HapticManager(this)

        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
        clipboardListener = ClipboardManager.OnPrimaryClipChangedListener {
            updateClipboardChip()
        }
        clipboard?.addPrimaryClipChangedListener(clipboardListener)
    }

    override fun onDestroy() {
        super.onDestroy()
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
        clipboardListener?.let { clipboard?.removePrimaryClipChangedListener(it) }
    }

    private fun updateClipboardChip() {
        if (!NativeBridge.isLibraryLoaded()) return
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager ?: return
        val clip = clipboard.primaryClip
        if (clip != null && clip.itemCount > 0) {
            val text = clip.getItemAt(0)?.coerceToText(this)?.toString()?.trim()
            if (!text.isNullOrEmpty() && text.length <= 1000) {
                NativeBridge.nativeSetClipboardText(text)
                keyboardView?.invalidate()
                return
            }
        }
        NativeBridge.nativeSetClipboardText(null)
        keyboardView?.invalidate()
    }

    override fun onConfigureWindow(win: Window, isFullscreen: Boolean, isCandidatesOnly: Boolean) {
        super.onConfigureWindow(win, isFullscreen, isCandidatesOnly)
        val prefs = getSharedPreferences("rynk_prefs", Context.MODE_PRIVATE)
        val themeId = prefs.getInt("theme_id", 1)
        applyWindowTheming(win, themeId)
    }

    override fun onCreateInputView(): View {
        val view = RynkKeyboardView(this)
        keyboardView = view

        val prefs = getSharedPreferences("rynk_prefs", Context.MODE_PRIVATE)
        val themeId = prefs.getInt("theme_id", 1)
        updateThemeAndWindowColors(themeId)
        applyLanguageAndProfanitySettings(prefs)

        view.setOnEventsReadyListener {
            pollAndDispatchOutputEvents()
        }

        return view
    }

    override fun onStartInputView(info: EditorInfo?, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        keyboardView?.resetState()
        updateClipboardChip()

        val prefs = getSharedPreferences("rynk_prefs", Context.MODE_PRIVATE)
        val themeId = prefs.getInt("theme_id", 1)
        hapticManager.isEnabled = prefs.getBoolean("pref_haptics", true)
        updateThemeAndWindowColors(themeId)
        applyLanguageAndProfanitySettings(prefs)
    }

    private fun getDefaultEnabledLanguages(): String {
        val sysLang = java.util.Locale.getDefault().language.lowercase()
        return when (sysLang) {
            "ru" -> "ru,en"
            "uk" -> "uk,en"
            "be" -> "be,ru,en"
            "kk" -> "kk,ru,en"
            "de" -> "de,en"
            "fr" -> "fr,en"
            "es" -> "es,en"
            "pt" -> "pt,en"
            "it" -> "it,en"
            "tr" -> "tr,en"
            else -> "en"
        }
    }

    private fun applyLanguageAndProfanitySettings(prefs: android.content.SharedPreferences) {
        if (!NativeBridge.isLibraryLoaded()) return
        val enabledLangs = prefs.getString("enabled_languages", null) ?: getDefaultEnabledLanguages()
        NativeBridge.nativeSetEnabledLanguages(enabledLangs)

        val profanityEnabled = prefs.getBoolean("pref_profanity", false)
        NativeBridge.nativeSetProfanityEnabled(profanityEnabled)

        val autocorrectEnabled = prefs.getBoolean("pref_autocorrect", true)
        NativeBridge.nativeSetAutocorrectEnabled(autocorrectEnabled)
    }

    override fun onUpdateSelection(
        oldSelStart: Int,
        oldSelEnd: Int,
        newSelStart: Int,
        newSelEnd: Int,
        candidatesStart: Int,
        candidatesEnd: Int
    ) {
        super.onUpdateSelection(oldSelStart, oldSelEnd, newSelStart, newSelEnd, candidatesStart, candidatesEnd)
        if (isDispatchingEvents) return
        if (oldSelStart != newSelStart || oldSelEnd != newSelEnd) {
            val ic = currentInputConnection
            val textBefore = ic?.getTextBeforeCursor(30, 0)?.toString() ?: ""
            if (textBefore.isEmpty() || textBefore.last().isWhitespace() || !textBefore.last().isLetterOrDigit()) {
                keyboardView?.resetState()
            }
        }
    }

    fun showRemoveWordDialog(word: String) {
        val builder = AlertDialog.Builder(this)
        builder.setTitle("${getString(R.string.btn_delete)} «$word»?")
        builder.setPositiveButton(getString(R.string.btn_delete)) { _, _ ->
            UserDictionaryManager(this).removeWord(word)
            NativeBridge.nativeRemoveUserWord(word)
            hapticManager.performHaptic(3)
            keyboardView?.invalidate()
        }
        builder.setNegativeButton(getString(R.string.btn_cancel), null)

        val dialog = builder.create()
        val window = dialog.window
        if (window != null) {
            val lp = window.attributes
            lp.token = keyboardView?.windowToken
            lp.type = WindowManager.LayoutParams.TYPE_APPLICATION_ATTACHED_DIALOG
            window.attributes = lp
            try {
                dialog.show()
                hapticManager.performHaptic(3)
            } catch (e: Exception) {
                // If attached dialog fails in this context, directly remove
                UserDictionaryManager(this).removeWord(word)
                NativeBridge.nativeRemoveUserWord(word)
                hapticManager.performHaptic(3)
                keyboardView?.invalidate()
            }
        }
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        if (keyCode == KeyEvent.KEYCODE_BACK && event.action == KeyEvent.ACTION_DOWN) {
            if (NativeBridge.isLibraryLoaded() && NativeBridge.nativeHandleBack()) {
                keyboardView?.invalidate()
                return true
            }
        }
        return super.onKeyDown(keyCode, event)
    }

    private fun updateThemeAndWindowColors(themeId: Int) {
        keyboardView?.applyTheme(themeId)
        window?.window?.let { win ->
            applyWindowTheming(win, themeId)
        }
    }

    private fun applyWindowTheming(win: Window, themeId: Int) {
        val bgColor = keyboardView?.getThemeBgColor(themeId) ?: when (themeId) {
            1 -> android.graphics.Color.rgb(238, 240, 245) // Light #EEF0F5
            2 -> android.graphics.Color.BLACK               // AMOLED
            3 -> android.graphics.Color.rgb(30, 22, 42)     // Sunset
            else -> android.graphics.Color.rgb(56, 58, 64)  // Soft Yandex Graphite #383A40
        }

        win.setBackgroundDrawable(ColorDrawable(bgColor))
        win.navigationBarColor = bgColor

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            win.isNavigationBarContrastEnforced = false
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val decor = win.decorView
            var flags = decor.systemUiVisibility
            flags = if (themeId == 1) { // Light theme needs dark navigation bar icons
                flags or View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR
            } else {
                flags and View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR.inv()
            }
            decor.systemUiVisibility = flags
        }
    }

    private fun pollAndDispatchOutputEvents() {
        if (!NativeBridge.isLibraryLoaded()) return

        val eventsString = NativeBridge.nativePollEvents()
        if (eventsString.isEmpty()) return

        val ic = currentInputConnection
        isDispatchingEvents = true
        ic?.beginBatchEdit()
        try {
            val lines = eventsString.split("\n")
            for (rawLine in lines) {
                val line = rawLine.removeSuffix("\r")
                if (line.isEmpty()) continue

                val firstTab = line.indexOf('\t')
                if (firstTab == -1) {
                    when (line.trim()) {
                        "SETTINGS" -> {
                            val intent = Intent(this, SettingsActivity::class.java).apply {
                                flags = Intent.FLAG_ACTIVITY_NEW_TASK
                            }
                            startActivity(intent)
                        }
                        "SWITCH_IME" -> {
                            val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
                            imm?.showInputMethodPicker()
                        }
                        "HIDE" -> {
                            requestHideSelf(0)
                        }
                        "DELETE_WORD" -> {
                            deletePreviousWord()
                        }
                    }
                    continue
                }

                val command = line.substring(0, firstTab)
                val payload = line.substring(firstTab + 1)
                when (command) {
                    "COMMIT" -> {
                        ic?.commitText(payload, 1)
                    }
                    "DELETE" -> {
                        val parts = payload.split("\t")
                        val before = parts.getOrNull(0)?.toIntOrNull() ?: 1
                        val after = parts.getOrNull(1)?.toIntOrNull() ?: 0
                        ic?.deleteSurroundingText(before, after)
                    }
                    "KEY" -> {
                        val keyCode = payload.toIntOrNull() ?: KeyEvent.KEYCODE_ENTER
                        ic?.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, keyCode))
                        ic?.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, keyCode))
                    }
                    "HAPTIC" -> {
                        val hapticCode = payload.toIntOrNull() ?: 1
                        hapticManager.performHaptic(hapticCode)
                    }
                    "CURSOR" -> {
                        val delta = payload.toIntOrNull() ?: 0
                        if (delta != 0) {
                            moveCursor(delta)
                        }
                    }
                }
            }
        } finally {
            ic?.endBatchEdit()
            isDispatchingEvents = false
        }
    }

    private fun moveCursor(delta: Int) {
        val ic = currentInputConnection ?: return
        if (delta < 0) {
            val count = -delta
            for (i in 0 until count) {
                ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_DPAD_LEFT))
                ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_DPAD_LEFT))
            }
        } else if (delta > 0) {
            for (i in 0 until delta) {
                ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_DPAD_RIGHT))
                ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_DPAD_RIGHT))
            }
        }
    }

    private fun deletePreviousWord() {
        val ic = currentInputConnection ?: return
        val textBefore = ic.getTextBeforeCursor(100, 0)?.toString() ?: ""
        if (textBefore.isEmpty()) {
            ic.deleteSurroundingText(1, 0)
            return
        }

        var i = textBefore.length - 1
        // Skip trailing spaces or punctuation
        while (i >= 0 && (textBefore[i].isWhitespace() || !textBefore[i].isLetterOrDigit())) {
            i--
        }
        // Skip letters of the word
        while (i >= 0 && textBefore[i].isLetterOrDigit()) {
            i--
        }

        val deleteCount = (textBefore.length - 1 - i).coerceAtLeast(1)
        ic.deleteSurroundingText(deleteCount, 0)
    }
}
