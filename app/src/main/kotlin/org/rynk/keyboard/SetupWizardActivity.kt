package org.rynk.keyboard

import android.content.Context
import android.content.Intent
import android.content.res.ColorStateList
import android.graphics.Color
import android.os.Bundle
import android.provider.Settings
import android.view.inputmethod.InputMethodManager
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton
import com.google.android.material.card.MaterialCardView

class SetupWizardActivity : AppCompatActivity() {

    private lateinit var cardStep1: MaterialCardView
    private lateinit var cardStep2: MaterialCardView
    private lateinit var btnEnableIme: MaterialButton
    private lateinit var btnSelectIme: MaterialButton
    private lateinit var btnOpenSettings: MaterialButton

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_setup_wizard)

        cardStep1 = findViewById(R.id.cardStep1)
        cardStep2 = findViewById(R.id.cardStep2)
        btnEnableIme = findViewById(R.id.btnEnableIme)
        btnSelectIme = findViewById(R.id.btnSelectIme)
        btnOpenSettings = findViewById(R.id.btnOpenSettings)

        btnEnableIme.setOnClickListener {
            // Open system input method settings
            startActivity(Intent(Settings.ACTION_INPUT_METHOD_SETTINGS))
        }

        btnSelectIme.setOnClickListener {
            // Open system input method picker
            val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
            imm?.showInputMethodPicker()
        }

        btnOpenSettings.setOnClickListener {
            startActivity(Intent(this, SettingsActivity::class.java))
        }
    }

    override fun onResume() {
        super.onResume()
        updateSetupSteps()
    }

    private fun updateSetupSteps() {
        val isEnabled = isKeyboardEnabled()
        val isSelected = isKeyboardSelected()

        val greenColor = Color.parseColor("#00E676")
        val cyanColor = Color.parseColor("#00D2FF")
        val mutedBg = Color.parseColor("#1C1F26")
        val activeText = Color.parseColor("#0C1016")

        if (isEnabled) {
            btnEnableIme.text = getString(R.string.step1_done) + " ✓"
            btnEnableIme.backgroundTintList = ColorStateList.valueOf(greenColor)
            btnEnableIme.setTextColor(activeText)
            cardStep1.strokeColor = greenColor
        } else {
            btnEnableIme.text = getString(R.string.step1_btn)
            btnEnableIme.backgroundTintList = ColorStateList.valueOf(cyanColor)
            btnEnableIme.setTextColor(activeText)
            cardStep1.strokeColor = cyanColor
        }

        if (isSelected) {
            btnSelectIme.text = getString(R.string.step2_done) + " ✓"
            btnSelectIme.backgroundTintList = ColorStateList.valueOf(greenColor)
            btnSelectIme.setTextColor(activeText)
            cardStep2.strokeColor = greenColor
        } else {
            btnSelectIme.text = getString(R.string.step2_btn)
            btnSelectIme.backgroundTintList = ColorStateList.valueOf(if (isEnabled) cyanColor else mutedBg)
            btnSelectIme.setTextColor(if (isEnabled) activeText else Color.WHITE)
            cardStep2.strokeColor = if (isEnabled) cyanColor else Color.TRANSPARENT
        }
    }

    private fun isKeyboardEnabled(): Boolean {
        val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager ?: return false
        val enabledList = imm.enabledInputMethodList
        return enabledList.any { it.packageName == packageName }
    }

    private fun isKeyboardSelected(): Boolean {
        val currentIme = Settings.Secure.getString(contentResolver, Settings.Secure.DEFAULT_INPUT_METHOD)
        return currentIme != null && currentIme.contains(packageName)
    }
}
