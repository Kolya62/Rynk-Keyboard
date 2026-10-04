package org.rynk.keyboard

import android.content.Context
import android.content.SharedPreferences
import java.util.Locale

/** All settings keys, defaults and their translation into engine values. */
object Prefs {
    const val FILE = "rynk_prefs"

    // Languages & gestures
    const val ENABLED_LANGUAGES = "enabled_languages"
    const val SPACE_SWIPE_LANGUAGE = "pref_space_swipe_language"
    const val BACKSPACE_SWIPE_WORD = "pref_backspace_swipe_word"
    const val DOUBLE_SPACE = "pref_double_space"

    // Typing & correction
    const val AUTOCORRECT_LEVEL = "pref_autocorrect_level"
    const val AUTO_CAPS = "pref_auto_caps"
    const val SMART_PUNCTUATION = "pref_smart_punctuation"
    const val NEXT_WORD = "pref_next_word"
    const val UNDO_AUTOCORRECT = "pref_undo_autocorrect"
    const val SPLIT_WORDS = "pref_split_words"
    const val PROFANITY = "pref_profanity"
    const val ADAPTIVE_LEARNING = "pref_adaptive_learning"

    // Appearance & feedback
    const val THEME = "theme_id"
    const val HEIGHT_PERCENT = "pref_height_percent"
    const val POPUP = "pref_popup"
    const val HAPTICS = "pref_haptics"
    const val HAPTIC_STRENGTH = "pref_haptic_strength"
    const val SOUND = "pref_sound"
    const val SOUND_VOLUME = "pref_sound_volume"

    // Layout
    const val NUMBER_ROW = "pref_number_row"
    const val ONE_HANDED = "pref_one_handed"
    const val VOICE_KEY = "pref_voice_key"

    // Clipboard
    const val CLIPBOARD_CHIP = "pref_clipboard_chip"
    const val CLIPBOARD_HISTORY = "pref_clipboard_history"
    const val CLIPBOARD_TTL = "pref_clipboard_ttl"

    /** Pre-settings-screen boolean, migrated to [AUTOCORRECT_LEVEL] */
    private const val LEGACY_AUTOCORRECT = "pref_autocorrect"

    const val AUTOCORRECT_OFF = 0
    const val AUTOCORRECT_NORMAL = 2

    // Engine flag bits, mirror of EngineSettings in rust-core/src/keyboard/state.rs
    private const val FLAG_AUTO_CAPS = 1
    private const val FLAG_SMART_PUNCTUATION = 1 shl 1
    private const val FLAG_NEXT_WORD = 1 shl 2
    private const val FLAG_UNDO_AUTOCORRECT = 1 shl 3
    private const val FLAG_SPACE_SWIPE_LANGUAGE = 1 shl 4
    private const val FLAG_BACKSPACE_SWIPE_WORD = 1 shl 5
    private const val FLAG_SPLIT_WORDS = 1 shl 6

    fun get(context: Context): SharedPreferences =
        context.getSharedPreferences(FILE, Context.MODE_PRIVATE).also { migrate(it) }

    internal fun migrate(prefs: SharedPreferences) {
        if (prefs.contains(LEGACY_AUTOCORRECT) && !prefs.contains(AUTOCORRECT_LEVEL)) {
            val level = if (prefs.getBoolean(LEGACY_AUTOCORRECT, true)) AUTOCORRECT_NORMAL else AUTOCORRECT_OFF
            prefs.edit().putString(AUTOCORRECT_LEVEL, level.toString()).remove(LEGACY_AUTOCORRECT).apply()
        }
    }

    fun defaultLanguages(): String {
        val sysLang = Locale.getDefault().language.lowercase()
        return when (sysLang) {
            "ru" -> "ru,en"
            "uk" -> "uk,en"
            "be" -> "be,ru,en"
            "kk" -> "kk,ru,en"
            "en" -> "en"
            else -> if (sysLang.length == 2) "$sysLang,en" else "en"
        }
    }

    fun enabledLanguages(prefs: SharedPreferences): String =
        prefs.getString(ENABLED_LANGUAGES, null) ?: defaultLanguages()

    /** 0 off, 1 mild, 2 normal, 3 aggressive (stored as a string for ListPreference) */
    fun autocorrectLevel(prefs: SharedPreferences): Int =
        prefs.getString(AUTOCORRECT_LEVEL, null)?.toIntOrNull() ?: AUTOCORRECT_NORMAL

    /** 0 switch language, 1 period, 2 nothing */
    fun doubleSpace(prefs: SharedPreferences): Int =
        prefs.getString(DOUBLE_SPACE, null)?.toIntOrNull() ?: 0

    fun engineFlags(prefs: SharedPreferences): Int {
        var flags = 0
        if (prefs.getBoolean(AUTO_CAPS, true)) flags = flags or FLAG_AUTO_CAPS
        if (prefs.getBoolean(SMART_PUNCTUATION, true)) flags = flags or FLAG_SMART_PUNCTUATION
        if (prefs.getBoolean(NEXT_WORD, true)) flags = flags or FLAG_NEXT_WORD
        if (prefs.getBoolean(UNDO_AUTOCORRECT, true)) flags = flags or FLAG_UNDO_AUTOCORRECT
        if (prefs.getBoolean(SPACE_SWIPE_LANGUAGE, true)) flags = flags or FLAG_SPACE_SWIPE_LANGUAGE
        if (prefs.getBoolean(BACKSPACE_SWIPE_WORD, true)) flags = flags or FLAG_BACKSPACE_SWIPE_WORD
        if (prefs.getBoolean(SPLIT_WORDS, true)) flags = flags or FLAG_SPLIT_WORDS
        return flags
    }

    fun theme(prefs: SharedPreferences): Int = prefs.getString(THEME_LIST, null)?.toIntOrNull()
        ?: prefs.getInt(THEME, 1)

    /** ListPreference stores strings; the keyboard historically read an int under [THEME] */
    const val THEME_LIST = "pref_theme"

    fun heightPercent(prefs: SharedPreferences): Int = prefs.getInt(HEIGHT_PERCENT, 100).coerceIn(80, 130)

    /** 1..100 */
    fun hapticStrength(prefs: SharedPreferences): Int = prefs.getInt(HAPTIC_STRENGTH, 60).coerceIn(1, 100)

    /** 0..100 */
    fun soundVolume(prefs: SharedPreferences): Int = prefs.getInt(SOUND_VOLUME, 40).coerceIn(0, 100)

    /** 0 off, 1 left, 2 right */
    fun oneHanded(prefs: SharedPreferences): Int = prefs.getString(ONE_HANDED, null)?.toIntOrNull() ?: 0

    /** How long unpinned clipboard history entries are kept */
    fun clipboardTtlMillis(prefs: SharedPreferences): Long =
        (prefs.getString(CLIPBOARD_TTL, null)?.toLongOrNull() ?: 60L) * 60_000L
}
