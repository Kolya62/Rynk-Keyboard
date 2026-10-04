package org.rynk.keyboard

import android.content.Context
import android.text.Editable
import android.text.TextWatcher
import android.widget.ArrayAdapter
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ListView
import androidx.appcompat.app.AlertDialog

/**
 * Multi-choice list of the 85 keyboard languages with a search field. Languages keep the order
 * they were enabled in, which is the order the keyboard cycles through them.
 */
object LanguagePickerDialog {

    fun show(context: Context, onChanged: () -> Unit) {
        val prefs = Prefs.get(context)
        val enabled = Prefs.enabledLanguages(prefs).split(",").map { it.trim() }.filter { it.isNotEmpty() }
            .toMutableList()

        val density = context.resources.displayMetrics.density
        val pad = (16 * density).toInt()
        val search = EditText(context).apply {
            hint = context.getString(R.string.dialog_languages_search)
            setSingleLine(true)
        }
        val list = ListView(context).apply { choiceMode = ListView.CHOICE_MODE_MULTIPLE }
        val container = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(pad, pad / 2, pad, 0)
            addView(search)
            addView(list, LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, (360 * density).toInt()))
        }

        var shown: List<Pair<String, String>> = emptyList()
        fun refresh() {
            val query = search.text.toString().trim().lowercase()
            shown = KeyboardLanguages.all.filter { query.isEmpty() || it.second.lowercase().contains(query) || it.first.contains(query) }
            list.adapter = ArrayAdapter(context, android.R.layout.simple_list_item_multiple_choice, shown.map { it.second })
            shown.forEachIndexed { i, lang -> list.setItemChecked(i, lang.first in enabled) }
        }
        list.setOnItemClickListener { _, _, position, _ ->
            val code = shown[position].first
            if (list.isItemChecked(position)) {
                if (code !in enabled) enabled.add(code)
            } else {
                enabled.remove(code)
            }
        }
        search.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}
            override fun afterTextChanged(s: Editable?) = refresh()
        })
        refresh()

        AlertDialog.Builder(context)
            .setTitle(R.string.dialog_languages_title)
            .setView(container)
            .setPositiveButton(android.R.string.ok) { _, _ ->
                if (enabled.isEmpty()) enabled.add("en")
                prefs.edit().putString(Prefs.ENABLED_LANGUAGES, enabled.joinToString(",")).apply()
                onChanged()
            }
            .setNegativeButton(R.string.btn_cancel, null)
            .show()
    }
}
