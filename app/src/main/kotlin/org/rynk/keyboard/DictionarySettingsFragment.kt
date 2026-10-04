package org.rynk.keyboard

import android.view.View
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.RadioButton
import android.widget.RadioGroup
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AlertDialog
import androidx.preference.Preference

/** The personal dictionary: add, browse and remove words, export and import as a text file. */
class DictionarySettingsFragment : RynkPreferenceFragment(R.xml.prefs_dictionary) {

    private val dictionary by lazy { UserDictionaryManager(requireContext()) }

    /** One word per line, optionally followed by a tab and "ru"/"en" */
    private val exportLauncher = registerForActivityResult(ActivityResultContracts.CreateDocument("text/plain")) { uri ->
        uri ?: return@registerForActivityResult
        val words = dictionary.getAllWords()
        val ok = runCatching {
            requireContext().contentResolver.openOutputStream(uri)?.bufferedWriter()?.use { out ->
                for (w in words) out.write("${w.word}\t${if (w.isRussian) "ru" else "en"}\n")
            } ?: error("no stream")
        }.isSuccess
        toast(if (ok) getString(R.string.words_exported, words.size) else getString(R.string.words_file_error))
    }

    private val importLauncher = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri ?: return@registerForActivityResult
        val lines = runCatching {
            requireContext().contentResolver.openInputStream(uri)?.bufferedReader()?.use { it.readLines() }
        }.getOrNull()
        if (lines == null) {
            toast(getString(R.string.words_file_error))
            return@registerForActivityResult
        }
        var count = 0
        for (line in lines) {
            val word = line.substringBefore('\t').trim()
            if (word.isEmpty() || word.length > 48 || word.any { it.isWhitespace() }) continue
            val tag = line.substringAfter('\t', "").trim().lowercase()
            val isRussian = if (tag.isNotEmpty()) tag == "ru" else word.any { it in 'Ѐ'..'ӿ' }
            dictionary.addWord(word, isRussian)
            count++
        }
        updateCount()
        toast(getString(R.string.words_imported, count))
    }

    override fun onPreferencesCreated() {
        updateCount()
        onClick("action_add_word") { showAddWordDialog() }
        onClick("action_view_words") { showWordsDialog() }
        onClick("action_export_words") { exportLauncher.launch("rynk-words.txt") }
        onClick("action_import_words") { importLauncher.launch(arrayOf("text/*")) }
    }

    private fun updateCount() {
        findPreference<Preference>("action_view_words")?.summary =
            getString(R.string.btn_view_words, dictionary.getAllWords().size)
    }

    private fun toast(text: String) = Toast.makeText(requireContext(), text, Toast.LENGTH_SHORT).show()

    private fun showAddWordDialog() {
        val context = requireContext()
        val pad = (18 * resources.displayMetrics.density).toInt()
        val input = EditText(context).apply {
            hint = getString(R.string.dialog_word_hint)
            setSingleLine(true)
        }
        val rbRu = RadioButton(context).apply {
            id = View.generateViewId()
            text = getString(R.string.lang_russian)
            isChecked = true
        }
        val rbEn = RadioButton(context).apply {
            id = View.generateViewId()
            text = getString(R.string.lang_english)
        }
        val container = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(pad, pad, pad, pad)
            addView(input)
            addView(RadioGroup(context).apply {
                orientation = RadioGroup.HORIZONTAL
                setPadding(0, pad / 2, 0, 0)
                addView(rbRu)
                addView(rbEn)
            })
        }
        AlertDialog.Builder(context)
            .setTitle(R.string.dialog_add_word_title)
            .setView(container)
            .setPositiveButton(R.string.btn_add) { _, _ ->
                val word = input.text.toString().trim()
                if (word.isNotEmpty()) {
                    dictionary.addWord(word, rbRu.isChecked)
                    updateCount()
                    toast(getString(R.string.word_added, word))
                }
            }
            .setNegativeButton(R.string.btn_cancel, null)
            .show()
    }

    private fun showWordsDialog() {
        val context = requireContext()
        val words = dictionary.getAllWords()
        if (words.isEmpty()) {
            AlertDialog.Builder(context)
                .setTitle(R.string.pref_user_dict_title)
                .setMessage(R.string.no_user_words)
                .setPositiveButton(android.R.string.ok, null)
                .show()
            return
        }
        val items = words.map { "${it.word}  (${if (it.isRussian) "RU" else "EN"})" }.toTypedArray()
        AlertDialog.Builder(context)
            .setTitle(R.string.pref_user_dict_title)
            .setItems(items) { _, which ->
                val target = words[which].word
                AlertDialog.Builder(context)
                    .setTitle("${getString(R.string.btn_delete)} «$target»?")
                    .setPositiveButton(R.string.btn_delete) { _, _ ->
                        dictionary.removeWord(target)
                        updateCount()
                    }
                    .setNegativeButton(R.string.btn_cancel, null)
                    .show()
            }
            .setPositiveButton(android.R.string.ok, null)
            .show()
    }
}
