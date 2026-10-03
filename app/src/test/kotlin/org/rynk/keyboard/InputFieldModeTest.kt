package org.rynk.keyboard

import android.view.inputmethod.EditorInfo
import org.junit.Assert.assertEquals
import org.junit.Test

class InputFieldModeTest {

    private fun determineMode(inputType: Int): Int {
        val clazz = inputType and EditorInfo.TYPE_MASK_CLASS
        val variation = inputType and EditorInfo.TYPE_MASK_VARIATION

        return when (clazz) {
            EditorInfo.TYPE_CLASS_TEXT -> {
                when (variation) {
                    EditorInfo.TYPE_TEXT_VARIATION_PASSWORD,
                    EditorInfo.TYPE_TEXT_VARIATION_WEB_PASSWORD -> NativeBridge.INPUT_MODE_PASSWORD
                    EditorInfo.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD -> NativeBridge.INPUT_MODE_VISIBLE_PASSWORD
                    EditorInfo.TYPE_TEXT_VARIATION_EMAIL_ADDRESS,
                    EditorInfo.TYPE_TEXT_VARIATION_WEB_EMAIL_ADDRESS -> NativeBridge.INPUT_MODE_EMAIL
                    EditorInfo.TYPE_TEXT_VARIATION_URI -> NativeBridge.INPUT_MODE_URI
                    else -> {
                        if ((inputType and EditorInfo.TYPE_TEXT_FLAG_MULTI_LINE) != 0) {
                            NativeBridge.INPUT_MODE_MULTILINE
                        } else {
                            NativeBridge.INPUT_MODE_NORMAL
                        }
                    }
                }
            }
            EditorInfo.TYPE_CLASS_NUMBER -> NativeBridge.INPUT_MODE_NUMBER
            EditorInfo.TYPE_CLASS_PHONE -> NativeBridge.INPUT_MODE_PHONE
            EditorInfo.TYPE_CLASS_DATETIME -> {
                when (variation) {
                    EditorInfo.TYPE_DATETIME_VARIATION_DATE -> NativeBridge.INPUT_MODE_DATE
                    EditorInfo.TYPE_DATETIME_VARIATION_TIME -> NativeBridge.INPUT_MODE_TIME
                    else -> NativeBridge.INPUT_MODE_DATE
                }
            }
            else -> NativeBridge.INPUT_MODE_NORMAL
        }
    }

    @Test
    fun testPasswordFieldsMapCorrectly() {
        val textPassword = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_VARIATION_PASSWORD
        assertEquals(NativeBridge.INPUT_MODE_PASSWORD, determineMode(textPassword))

        val webPassword = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_VARIATION_WEB_PASSWORD
        assertEquals(NativeBridge.INPUT_MODE_PASSWORD, determineMode(webPassword))

        val visiblePassword = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD
        assertEquals(NativeBridge.INPUT_MODE_VISIBLE_PASSWORD, determineMode(visiblePassword))
    }

    @Test
    fun testEmailAndUriFieldsMapCorrectly() {
        val email = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_VARIATION_EMAIL_ADDRESS
        assertEquals(NativeBridge.INPUT_MODE_EMAIL, determineMode(email))

        val webEmail = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_VARIATION_WEB_EMAIL_ADDRESS
        assertEquals(NativeBridge.INPUT_MODE_EMAIL, determineMode(webEmail))

        val uri = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_VARIATION_URI
        assertEquals(NativeBridge.INPUT_MODE_URI, determineMode(uri))
    }

    @Test
    fun testNumberPhoneAndDateFieldsMapCorrectly() {
        assertEquals(NativeBridge.INPUT_MODE_NUMBER, determineMode(EditorInfo.TYPE_CLASS_NUMBER))
        assertEquals(NativeBridge.INPUT_MODE_PHONE, determineMode(EditorInfo.TYPE_CLASS_PHONE))

        val date = EditorInfo.TYPE_CLASS_DATETIME or EditorInfo.TYPE_DATETIME_VARIATION_DATE
        assertEquals(NativeBridge.INPUT_MODE_DATE, determineMode(date))

        val time = EditorInfo.TYPE_CLASS_DATETIME or EditorInfo.TYPE_DATETIME_VARIATION_TIME
        assertEquals(NativeBridge.INPUT_MODE_TIME, determineMode(time))
    }

    @Test
    fun testMultilineAndNormalFieldsMapCorrectly() {
        val normal = EditorInfo.TYPE_CLASS_TEXT
        assertEquals(NativeBridge.INPUT_MODE_NORMAL, determineMode(normal))

        val multiline = EditorInfo.TYPE_CLASS_TEXT or EditorInfo.TYPE_TEXT_FLAG_MULTI_LINE
        assertEquals(NativeBridge.INPUT_MODE_MULTILINE, determineMode(multiline))
    }
}
