package org.rynk.keyboard

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.view.View
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.google.android.material.appbar.MaterialToolbar

/**
 * Settings: a root list of sections, each a preference screen. Values are stored in
 * [Prefs.FILE]; the running keyboard picks changes up through its preference listener.
 */
class SettingsActivity : AppCompatActivity(), PreferenceFragmentCompat.OnPreferenceStartFragmentCallback {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, false)
        setContentView(R.layout.activity_settings)

        val root = findViewById<View>(R.id.settingsRoot)
        val toolbar = findViewById<MaterialToolbar>(R.id.toolbar)
        val container = findViewById<View>(R.id.settingsContainer)

        setSupportActionBar(toolbar)
        supportActionBar?.setDisplayHomeAsUpEnabled(true)
        toolbar.setNavigationOnClickListener { onBackPressedDispatcher.onBackPressed() }

        ViewCompat.setOnApplyWindowInsetsListener(root) { _, windowInsets ->
            val insets = windowInsets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            toolbar.updatePadding(top = insets.top, left = insets.left, right = insets.right)
            container.updatePadding(left = insets.left, right = insets.right, bottom = insets.bottom)
            windowInsets
        }

        if (savedInstanceState == null) {
            supportFragmentManager.beginTransaction()
                .replace(R.id.settingsContainer, RootSettingsFragment())
                .commit()
        }
        supportFragmentManager.addOnBackStackChangedListener { updateTitle() }
        updateTitle()

        checkAndShowSupportDialog()
    }

    private fun updateTitle() {
        val entry = supportFragmentManager.backStackEntryCount
        supportActionBar?.title = if (entry == 0) {
            getString(R.string.settings_title)
        } else {
            supportFragmentManager.getBackStackEntryAt(entry - 1).name
        }
    }

    override fun onPreferenceStartFragment(caller: PreferenceFragmentCompat, pref: Preference): Boolean {
        val fragment = supportFragmentManager.fragmentFactory
            .instantiate(classLoader, pref.fragment ?: return false)
        supportFragmentManager.beginTransaction()
            .replace(R.id.settingsContainer, fragment)
            .addToBackStack(pref.title?.toString())
            .commit()
        return true
    }

    private fun checkAndShowSupportDialog() {
        val prefs = Prefs.get(this)
        if (!prefs.getBoolean("has_shown_support_dialog", false)) {
            prefs.edit().putBoolean("has_shown_support_dialog", true).apply()
            AlertDialog.Builder(this)
                .setTitle(getString(R.string.support_dialog_title))
                .setMessage(getString(R.string.support_dialog_message))
                .setPositiveButton(getString(R.string.btn_copy_card)) { _, _ -> copyCardNumber(this) }
                .setNegativeButton(getString(R.string.btn_close), null)
                .show()
        }
    }

    override fun onSupportNavigateUp(): Boolean {
        onBackPressedDispatcher.onBackPressed()
        return true
    }

    companion object {
        fun copyCardNumber(context: Context) {
            val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            clipboard.setPrimaryClip(ClipData.newPlainText("Rynk Card Number", context.getString(R.string.support_card_number)))
            Toast.makeText(context, context.getString(R.string.card_copied), Toast.LENGTH_SHORT).show()
        }
    }
}

/** Base for the settings sections: all of them read and write the keyboard's preference file. */
abstract class RynkPreferenceFragment(private val xml: Int) : PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        preferenceManager.sharedPreferencesName = Prefs.FILE
        Prefs.get(requireContext())
        setPreferencesFromResource(xml, rootKey)
        onPreferencesCreated()
    }

    open fun onPreferencesCreated() {}

    protected fun onClick(key: String, action: () -> Unit) {
        findPreference<Preference>(key)?.setOnPreferenceClickListener { action(); true }
    }
}

class RootSettingsFragment : RynkPreferenceFragment(R.xml.prefs_root)

class TypingSettingsFragment : RynkPreferenceFragment(R.xml.prefs_typing)

class AppearanceSettingsFragment : RynkPreferenceFragment(R.xml.prefs_appearance) {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        // Carry the theme chosen with the old settings screen over to the list preference
        val prefs = Prefs.get(requireContext())
        if (!prefs.contains(Prefs.THEME_LIST)) {
            prefs.edit().putString(Prefs.THEME_LIST, prefs.getInt(Prefs.THEME, 1).toString()).apply()
        }
        super.onCreatePreferences(savedInstanceState, rootKey)
    }
}

class LanguagesSettingsFragment : RynkPreferenceFragment(R.xml.prefs_languages) {
    override fun onPreferencesCreated() {
        updateSummary()
        onClick("action_languages") {
            LanguagePickerDialog.show(requireContext()) { updateSummary() }
        }
    }

    private fun updateSummary() {
        val codes = Prefs.enabledLanguages(Prefs.get(requireContext())).split(",").filter { it.isNotBlank() }
        findPreference<Preference>("action_languages")?.summary =
            codes.joinToString(", ") { KeyboardLanguages.name(it.trim()).substringBefore(" (") }
    }
}

class PrivacySettingsFragment : RynkPreferenceFragment(R.xml.prefs_privacy) {
    override fun onPreferencesCreated() {
        onClick("action_clear_learned") {
            AlertDialog.Builder(requireContext())
                .setTitle(R.string.btn_clear_adaptive_data)
                .setPositiveButton(R.string.btn_delete) { _, _ ->
                    if (NativeBridge.isCoreInitialized) NativeBridge.nativeClearAdaptiveData()
                    runCatching { java.io.File(requireContext().filesDir, "adaptive_dict.bin").delete() }
                    Toast.makeText(requireContext(), R.string.adaptive_data_cleared, Toast.LENGTH_SHORT).show()
                }
                .setNegativeButton(R.string.btn_cancel, null)
                .show()
        }
    }
}

class AboutSettingsFragment : RynkPreferenceFragment(R.xml.prefs_about) {
    override fun onPreferencesCreated() {
        val context = requireContext()
        findPreference<Preference>("about_version")?.summary = runCatching {
            context.packageManager.getPackageInfo(context.packageName, 0).versionName
        }.getOrNull()
        onClick("action_licenses") {
            AlertDialog.Builder(context)
                .setTitle(R.string.pref_licenses)
                .setMessage(R.string.licenses_text)
                .setPositiveButton(R.string.btn_close, null)
                .show()
        }
        onClick("action_support") { SettingsActivity.copyCardNumber(context) }
    }
}
