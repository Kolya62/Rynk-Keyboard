package org.rynk.keyboard

import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.RadioButton
import android.widget.RadioGroup
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import android.view.View
import android.widget.ScrollView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding
import com.google.android.material.appbar.MaterialToolbar
import com.google.android.material.button.MaterialButton
import com.google.android.material.switchmaterial.SwitchMaterial

class SettingsActivity : AppCompatActivity() {

    private lateinit var dictManager: UserDictionaryManager
    private lateinit var btnViewUserWords: MaterialButton

    private val allLanguages = listOf(
        "ru" to "Русский (Russian)",
        "en" to "English",
        "de" to "Deutsch (German)",
        "fr" to "Français (French)",
        "es" to "Español (Spanish)",
        "pt" to "Português (Portuguese)",
        "it" to "Italiano (Italian)",
        "tr" to "Türkçe (Turkish)",
        "uk" to "Українська (Ukrainian)",
        "be" to "Беларуская (Belarusian)",
        "kk" to "Қазақша (Kazakh)",
        "ar" to "العربية (Arabic)",
        "pl" to "Polski (Polish)",
        "cs" to "Čeština (Czech)",
        "ro" to "Română (Romanian)",
        "nl" to "Nederlands (Dutch)",
        "sv" to "Svenska (Swedish)",
        "no" to "Norsk (Norwegian)",
        "da" to "Dansk (Danish)",
        "fi" to "Suomi (Finnish)",
        "el" to "Ελληνικά (Greek)",
        "he" to "עברית (Hebrew)",
        "fa" to "فارسی (Persian)",
        "ur" to "اردو (Urdu)",
        "hi" to "हिन्दी (Hindi)",
        "bn" to "বাংলা (Bengali)",
        "id" to "Bahasa Indonesia (Indonesian)",
        "ms" to "Bahasa Melayu (Malay)",
        "vi" to "Tiếng Việt (Vietnamese)",
        "th" to "ไทย (Thai)",
        "hu" to "Magyar (Hungarian)",
        "bg" to "Български (Bulgarian)",
        "sr" to "Српски (Serbian)",
        "hr" to "Hrvatski (Croatian)",
        "sk" to "Slovenčina (Slovak)",
        "sl" to "Slovenščina (Slovenian)",
        "lt" to "Lietuvių (Lithuanian)",
        "lv" to "Latviešu (Latvian)",
        "et" to "Eesti (Estonian)",
        "ka" to "ქართული (Georgian)",
        "hy" to "Հայերեն (Armenian)",
        "az" to "Azərbaycan (Azerbaijani)",
        "uz" to "Oʻzbekcha (Uzbek)",
        "tg" to "Тоҷикӣ (Tajik)",
        "ky" to "Кыргызча (Kyrgyz)",
        "tk" to "Türkmençe (Turkmen)",
        "mn" to "Монгол (Mongolian)",
        "tl" to "Tagalog (Filipino)",
        "sq" to "Shqip (Albanian)",
        "bs" to "Bosanski (Bosnian)",
        "mk" to "Македонски (Macedonian)",
        "is" to "Íslenska (Icelandic)",
        "ga" to "Gaeilge (Irish)",
        "cy" to "Cymraeg (Welsh)",
        "eu" to "Euskara (Basque)",
        "ca" to "Català (Catalan)",
        "gl" to "Galego (Galician)",
        "af" to "Afrikaans",
        "sw" to "Kiswahili (Swahili)",
        "ha" to "Hausa",
        "yo" to "Yorùbá (Yoruba)",
        "ig" to "Igbo",
        "zu" to "isiZulu (Zulu)",
        "eo" to "Esperanto",
        "la" to "Latina (Latin)",
        "ta" to "தமிழ் (Tamil)",
        "te" to "తెలుగు (Telugu)",
        "mr" to "मराठी (Marathi)",
        "gu" to "ગુજરાતી (Gujarati)",
        "kn" to "ಕನ್ನಡ (Kannada)",
        "ml" to "മലയാളം (Malayalam)",
        "pa" to "ਪੰਜਾਬੀ (Punjabi)",
        "ne" to "नेपाली (Nepali)",
        "si" to "සිංහල (Sinhala)",
        "my" to "မြန်မာ (Burmese)",
        "km" to "ភាសាខ្មែរ (Khmer)",
        "am" to "አማርኛ (Amharic)",
        "so" to "Soomaali (Somali)",
        "ku" to "Kurdî (Kurdish)",
        "mt" to "Malti (Maltese)"
    )

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, false)
        setContentView(R.layout.activity_settings)

        val root = findViewById<View>(R.id.settingsRoot)
        val toolbar = findViewById<MaterialToolbar>(R.id.toolbar)
        val scrollView = findViewById<ScrollView>(R.id.settingsScrollView)

        setSupportActionBar(toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        supportActionBar?.title = getString(R.string.settings_title)
        toolbar.setNavigationOnClickListener { finish() }

        ViewCompat.setOnApplyWindowInsetsListener(root) { _, windowInsets ->
            val insets = windowInsets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            // Lower top part taking into account status bar and camera cutout
            toolbar.updatePadding(
                top = insets.top,
                left = insets.left,
                right = insets.right
            )
            scrollView.updatePadding(
                left = insets.left,
                right = insets.right,
                bottom = insets.bottom + (16 * resources.displayMetrics.density).toInt()
            )
            windowInsets
        }

        val prefs = getSharedPreferences("rynk_prefs", Context.MODE_PRIVATE)
        dictManager = UserDictionaryManager(this)

        val rgThemes = findViewById<RadioGroup>(R.id.rgThemes)
        val rbDark = findViewById<RadioButton>(R.id.rbDark)
        val rbLight = findViewById<RadioButton>(R.id.rbLight)
        val rbAmoled = findViewById<RadioButton>(R.id.rbAmoled)
        val rbSunset = findViewById<RadioButton>(R.id.rbSunset)

        val switchAutocorrect = findViewById<SwitchMaterial>(R.id.switchAutocorrect)
        val switchPopup = findViewById<SwitchMaterial>(R.id.switchPopup)
        val switchProfanity = findViewById<SwitchMaterial>(R.id.switchProfanity)
        val btnKeyboardLanguages = findViewById<MaterialButton>(R.id.btnKeyboardLanguages)
        val switchHaptics = findViewById<SwitchMaterial>(R.id.switchHaptics)
        val switchAdaptiveLearning = findViewById<SwitchMaterial>(R.id.switchAdaptiveLearning)
        val btnClearAdaptiveData = findViewById<MaterialButton>(R.id.btnClearAdaptiveData)

        val btnAddUserWord = findViewById<MaterialButton>(R.id.btnAddUserWord)
        btnViewUserWords = findViewById(R.id.btnViewUserWords)

        // Default to Light theme (1)
        val currentThemeId = prefs.getInt("theme_id", 1)
        when (currentThemeId) {
            0 -> rbDark.isChecked = true
            1 -> rbLight.isChecked = true
            2 -> rbAmoled.isChecked = true
            3 -> rbSunset.isChecked = true
            else -> rbLight.isChecked = true
        }

        switchAutocorrect.isChecked = prefs.getBoolean("pref_autocorrect", true)
        switchPopup.isChecked = prefs.getBoolean("pref_popup", true)
        switchProfanity.isChecked = prefs.getBoolean("pref_profanity", false)
        switchHaptics.isChecked = prefs.getBoolean("pref_haptics", true)
        switchAdaptiveLearning.isChecked = prefs.getBoolean("pref_adaptive_learning", true)

        rgThemes.setOnCheckedChangeListener { _, checkedId ->
            val themeId = when (checkedId) {
                R.id.rbDark -> 0
                R.id.rbAmoled -> 2
                R.id.rbSunset -> 3
                else -> 1 // Default to Light
            }
            prefs.edit().putInt("theme_id", themeId).apply()
            if (NativeBridge.isLibraryLoaded()) {
                NativeBridge.nativeSetTheme(themeId)
            }
        }

        switchAutocorrect.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean("pref_autocorrect", isChecked).apply()
            if (NativeBridge.isLibraryLoaded()) {
                NativeBridge.nativeSetAutocorrectEnabled(isChecked)
            }
        }

        switchPopup.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean("pref_popup", isChecked).apply()
            if (NativeBridge.isLibraryLoaded()) {
                NativeBridge.nativeSetPopupEnabled(isChecked)
            }
        }

        switchProfanity.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean("pref_profanity", isChecked).apply()
            if (NativeBridge.isLibraryLoaded()) {
                NativeBridge.nativeSetProfanityEnabled(isChecked)
            }
        }

        btnKeyboardLanguages.setOnClickListener {
            showLanguagesDialog()
        }

        switchHaptics.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean("pref_haptics", isChecked).apply()
        }

        switchAdaptiveLearning.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean("pref_adaptive_learning", isChecked).apply()
            if (NativeBridge.isLibraryLoaded()) {
                NativeBridge.nativeSetAdaptiveLearningEnabled(isChecked)
            }
        }

        btnClearAdaptiveData.setOnClickListener {
            if (NativeBridge.isLibraryLoaded()) {
                NativeBridge.nativeClearAdaptiveData()
            }
            try {
                val file = java.io.File(filesDir, "adaptive_dict.bin")
                if (file.exists()) {
                    file.delete()
                }
            } catch (ignored: Exception) {}
            Toast.makeText(this, getString(R.string.adaptive_data_cleared), Toast.LENGTH_SHORT).show()
        }

        updateUserWordCount()

        btnAddUserWord.setOnClickListener {
            showAddWordDialog()
        }

        btnViewUserWords.setOnClickListener {
            showViewWordsDialog()
        }
    }

    private fun getDefaultEnabledLanguages(): String {
        val sysLang = java.util.Locale.getDefault().language.lowercase()
        return when (sysLang) {
            "ru" -> "ru,en"
            "uk" -> "uk,en"
            "be" -> "be,ru,en"
            "kk" -> "kk,ru,en"
            "en" -> "en"
            else -> if (sysLang.length == 2) "$sysLang,en" else "en"
        }
    }

    private fun showLanguagesDialog() {
        val prefs = getSharedPreferences("rynk_prefs", Context.MODE_PRIVATE)
        val currentStr = prefs.getString("enabled_languages", null) ?: getDefaultEnabledLanguages()
        val currentSet = currentStr.split(",").map { it.trim().lowercase() }.toMutableSet()

        val names = allLanguages.map { it.second }.toTypedArray()
        val checkedItems = BooleanArray(allLanguages.size) { i ->
            currentSet.contains(allLanguages[i].first)
        }

        AlertDialog.Builder(this)
            .setTitle(getString(R.string.dialog_languages_title))
            .setMultiChoiceItems(names, checkedItems) { _, which, isChecked ->
                val code = allLanguages[which].first
                if (isChecked) {
                    currentSet.add(code)
                } else {
                    currentSet.remove(code)
                }
            }
            .setPositiveButton("OK") { _, _ ->
                if (currentSet.isEmpty()) {
                    currentSet.add("en")
                }
                val result = allLanguages.map { it.first }.filter { currentSet.contains(it) }.joinToString(",")
                prefs.edit().putString("enabled_languages", result).apply()
                if (NativeBridge.isLibraryLoaded()) {
                    NativeBridge.nativeSetEnabledLanguages(result)
                }
            }
            .setNegativeButton(getString(R.string.btn_cancel), null)
            .show()
    }

    private fun updateUserWordCount() {
        val count = dictManager.getAllWords().size
        btnViewUserWords.text = getString(R.string.btn_view_words, count)
    }

    private fun showAddWordDialog() {
        val container = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            val pad = (18 * resources.displayMetrics.density).toInt()
            setPadding(pad, pad, pad, pad)
        }

        val input = EditText(this).apply {
            hint = getString(R.string.dialog_word_hint)
            setSingleLine(true)
        }
        container.addView(input)

        val rgLang = RadioGroup(this).apply {
            orientation = RadioGroup.HORIZONTAL
            val padTop = (12 * resources.displayMetrics.density).toInt()
            setPadding(0, padTop, 0, 0)
        }
        val rbRu = RadioButton(this).apply {
            id = View.generateViewId()
            text = getString(R.string.lang_russian)
            isChecked = true
        }
        val rbEn = RadioButton(this).apply {
            id = View.generateViewId()
            text = getString(R.string.lang_english)
        }
        rgLang.addView(rbRu)
        rgLang.addView(rbEn)
        container.addView(rgLang)

        AlertDialog.Builder(this)
            .setTitle(getString(R.string.dialog_add_word_title))
            .setView(container)
            .setPositiveButton(getString(R.string.btn_add)) { _, _ ->
                val word = input.text.toString().trim()
                if (word.isNotEmpty()) {
                    val isRu = rbRu.isChecked
                    dictManager.addWord(word, isRu)
                    updateUserWordCount()
                    Toast.makeText(this, "«$word» добавлено в словарь", Toast.LENGTH_SHORT).show()
                }
            }
            .setNegativeButton(getString(R.string.btn_cancel), null)
            .show()
    }

    private fun showViewWordsDialog() {
        val words = dictManager.getAllWords()
        if (words.isEmpty()) {
            AlertDialog.Builder(this)
                .setTitle(getString(R.string.pref_user_dict_title))
                .setMessage(getString(R.string.no_user_words))
                .setPositiveButton("OK", null)
                .show()
            return
        }

        val displayItems = words.map { "${it.word}  (${if (it.isRussian) "RU" else "EN"})" }.toTypedArray()

        AlertDialog.Builder(this)
            .setTitle(getString(R.string.pref_user_dict_title))
            .setItems(displayItems) { _, which ->
                val targetWord = words[which].word
                AlertDialog.Builder(this)
                    .setTitle("${getString(R.string.btn_delete)} «$targetWord»?")
                    .setPositiveButton(getString(R.string.btn_delete)) { _, _ ->
                        dictManager.removeWord(targetWord)
                        updateUserWordCount()
                    }
                    .setNegativeButton(getString(R.string.btn_cancel), null)
                    .show()
            }
            .setPositiveButton("OK", null)
            .show()
    }

    override fun onSupportNavigateUp(): Boolean {
        finish()
        return true
    }
}
