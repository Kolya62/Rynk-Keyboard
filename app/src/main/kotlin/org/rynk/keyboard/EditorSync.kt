package org.rynk.keyboard

import android.view.inputmethod.EditorInfo

/**
 * Pure helpers that decide how the engine's view of the text relates to the editor.
 * Kept free of Android framework state so they can be unit-tested.
 */
object EditorSync {

    /** Trailing run of letters/digits right before the cursor. */
    fun trailingWord(textBeforeCursor: CharSequence): String {
        var i = textBeforeCursor.length
        while (i > 0 && textBeforeCursor[i - 1].isLetterOrDigit()) i--
        return textBeforeCursor.subSequence(i, textBeforeCursor.length).toString()
    }

    /**
     * Whether the engine's composing word still matches the editor after a selection update.
     *
     * Commits are asynchronous, so the editor may lag a few characters behind the engine
     * (or be ahead of it while deletions are in flight). Being a prefix in either direction
     * counts as "in sync"; anything else means the user moved the cursor and the engine
     * must forget its composing word before autocorrect deletes the wrong text.
     */
    fun isEngineInSync(textBeforeCursor: CharSequence, composing: String, lastCommittedWord: String): Boolean {
        val word = trailingWord(textBeforeCursor).lowercase()
        if (composing.isEmpty()) {
            return word.isEmpty() || word == lastCommittedWord.lowercase()
        }
        val comp = composing.lowercase()
        if (word.isEmpty()) {
            // Only the very first character of a word can still be in flight
            return comp.length == 1
        }
        return comp.startsWith(word) || word.startsWith(comp)
    }

    /**
     * Editor action the Enter key should perform, or 0 for a plain Enter / newline.
     * Mirrors the framework convention: IME_FLAG_NO_ENTER_ACTION (set by multi-line
     * TextViews) and the NONE/UNSPECIFIED actions mean "insert a newline".
     */
    fun enterAction(imeOptions: Int): Int {
        if ((imeOptions and EditorInfo.IME_FLAG_NO_ENTER_ACTION) != 0) return 0
        return when (val action = imeOptions and EditorInfo.IME_MASK_ACTION) {
            EditorInfo.IME_ACTION_UNSPECIFIED, EditorInfo.IME_ACTION_NONE -> 0
            else -> action
        }
    }

    /**
     * Length (in UTF-16 units) of the last [words] words before the cursor, including the
     * spaces and punctuation after each word: what a backspace drag selects for deletion.
     */
    fun wordsBackLength(textBeforeCursor: CharSequence, words: Int): Int {
        var i = textBeforeCursor.length
        repeat(words) {
            while (i > 0 && !textBeforeCursor[i - 1].isLetterOrDigit()) i--
            while (i > 0 && textBeforeCursor[i - 1].isLetterOrDigit()) i--
        }
        return textBeforeCursor.length - i
    }
}
