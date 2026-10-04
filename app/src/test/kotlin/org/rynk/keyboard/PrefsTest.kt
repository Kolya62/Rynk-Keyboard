package org.rynk.keyboard

import android.content.SharedPreferences
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class PrefsTest {

    /** Minimal in-memory SharedPreferences for JVM tests */
    private class MemoryPrefs(initial: Map<String, Any?> = emptyMap()) : SharedPreferences {
        val values = initial.toMutableMap()
        override fun getAll(): MutableMap<String, *> = values
        override fun getString(key: String, defValue: String?) = values[key] as? String ?: defValue
        override fun getStringSet(key: String, defValues: MutableSet<String>?) = defValues
        override fun getInt(key: String, defValue: Int) = values[key] as? Int ?: defValue
        override fun getLong(key: String, defValue: Long) = values[key] as? Long ?: defValue
        override fun getFloat(key: String, defValue: Float) = values[key] as? Float ?: defValue
        override fun getBoolean(key: String, defValue: Boolean) = values[key] as? Boolean ?: defValue
        override fun contains(key: String) = values.containsKey(key)
        override fun registerOnSharedPreferenceChangeListener(l: SharedPreferences.OnSharedPreferenceChangeListener?) {}
        override fun unregisterOnSharedPreferenceChangeListener(l: SharedPreferences.OnSharedPreferenceChangeListener?) {}
        override fun edit(): SharedPreferences.Editor = object : SharedPreferences.Editor {
            override fun putString(key: String, value: String?) = apply { values[key] = value }
            override fun putStringSet(key: String, values: MutableSet<String>?) = this
            override fun putInt(key: String, value: Int) = apply { this@MemoryPrefs.values[key] = value }
            override fun putLong(key: String, value: Long) = apply { this@MemoryPrefs.values[key] = value }
            override fun putFloat(key: String, value: Float) = apply { this@MemoryPrefs.values[key] = value }
            override fun putBoolean(key: String, value: Boolean) = apply { this@MemoryPrefs.values[key] = value }
            override fun remove(key: String) = apply { values.remove(key) }
            override fun clear() = apply { values.clear() }
            override fun commit() = true
            override fun apply() {}
        }
    }

    @Test
    fun defaultsEnableEverythingExceptNothing() {
        val prefs = MemoryPrefs()
        assertEquals(0b111111111, Prefs.engineFlags(prefs))
        assertEquals(Prefs.AUTOCORRECT_NORMAL, Prefs.autocorrectLevel(prefs))
        assertEquals(0, Prefs.doubleSpace(prefs))
        assertEquals(100, Prefs.heightPercent(prefs))
    }

    @Test
    fun switchesClearTheirFlagBits() {
        val prefs = MemoryPrefs(mapOf(Prefs.AUTO_CAPS to false, Prefs.SPLIT_WORDS to false))
        val flags = Prefs.engineFlags(prefs)
        assertEquals(0, flags and 1)
        assertEquals(0, flags and (1 shl 6))
        assertEquals(1 shl 1, flags and (1 shl 1))
    }

    @Test
    fun themeListOverridesLegacyInt() {
        assertEquals(2, Prefs.theme(MemoryPrefs(mapOf(Prefs.THEME to 2))))
        assertEquals(3, Prefs.theme(MemoryPrefs(mapOf(Prefs.THEME to 2, Prefs.THEME_LIST to "3"))))
    }

    @Test
    fun heightIsClamped() {
        assertEquals(130, Prefs.heightPercent(MemoryPrefs(mapOf(Prefs.HEIGHT_PERCENT to 400))))
        assertEquals(80, Prefs.heightPercent(MemoryPrefs(mapOf(Prefs.HEIGHT_PERCENT to 10))))
    }

    @Test
    fun legacyAutocorrectSwitchMigrates() {
        val prefs = MemoryPrefs(mapOf("pref_autocorrect" to false))
        Prefs.migrate(prefs)
        assertEquals(Prefs.AUTOCORRECT_OFF, Prefs.autocorrectLevel(prefs))
        assertFalse(prefs.contains("pref_autocorrect"))
    }
}
