package org.rynk.keyboard

import android.content.Context
import android.content.SharedPreferences

data class UserWord(val word: String, val isRussian: Boolean)

class UserDictionaryManager(context: Context) {

    private val prefs: SharedPreferences =
        context.getSharedPreferences("rynk_user_dictionary", Context.MODE_PRIVATE)

    fun getAllWords(): List<UserWord> {
        val allEntries = prefs.all
        val list = mutableListOf<UserWord>()
        for ((key, value) in allEntries) {
            if (key == "forgotten_words") continue
            val isRu = (value as? Boolean) ?: true
            list.add(UserWord(key, isRu))
        }
        return list.sortedBy { it.word }
    }

    fun addWord(word: String, isRussian: Boolean) {
        val clean = word.trim().lowercase()
        if (clean.isEmpty()) return
        val forgotten = prefs.getStringSet("forgotten_words", null)
        if (forgotten != null && forgotten.contains(clean)) {
            val updated = HashSet(forgotten)
            updated.remove(clean)
            prefs.edit().putStringSet("forgotten_words", updated).apply()
        }
        prefs.edit().putBoolean(clean, isRussian).apply()
        if (NativeBridge.isLibraryLoaded()) {
            NativeBridge.nativeAddUserWord(clean, isRussian)
        }
    }

    fun removeWord(word: String) {
        val clean = word.trim().lowercase()
        if (clean.isEmpty()) return
        prefs.edit().remove(clean).apply()

        // Also add to forgotten_words set so it stays blocked in suggestions
        val forgotten = prefs.getStringSet("forgotten_words", null) ?: emptySet()
        val updated = HashSet(forgotten)
        updated.add(clean)
        prefs.edit().putStringSet("forgotten_words", updated).apply()

        if (NativeBridge.isLibraryLoaded()) {
            NativeBridge.nativeRemoveUserWord(clean)
        }
    }

    fun syncToNative() {
        if (!NativeBridge.isLibraryLoaded()) return
        val allEntries = prefs.all
        for ((key, value) in allEntries) {
            if (key == "forgotten_words") continue
            val isRu = (value as? Boolean) ?: true
            NativeBridge.nativeAddUserWord(key, isRu)
        }
        val forgotten = prefs.getStringSet("forgotten_words", null)
        if (forgotten != null) {
            for (w in forgotten) {
                NativeBridge.nativeRemoveUserWord(w)
            }
        }
    }
}
