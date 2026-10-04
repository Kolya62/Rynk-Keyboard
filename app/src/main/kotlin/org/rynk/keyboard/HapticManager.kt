package org.rynk.keyboard

import android.content.Context
import android.media.AudioManager
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager

/** Key press feedback: vibration (with adjustable strength) and the system click sound. */
class HapticManager(context: Context) {

    private val vibrator: Vibrator? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val manager = context.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as? VibratorManager
        manager?.defaultVibrator
    } else {
        @Suppress("DEPRECATION")
        context.getSystemService(Context.VIBRATOR_SERVICE) as? Vibrator
    }
    private val audio = context.getSystemService(Context.AUDIO_SERVICE) as? AudioManager

    var isEnabled: Boolean = true
    /** 1..100, 60 = the default feel */
    var strength: Int = 60
    var soundEnabled: Boolean = false
    /** 0..1 */
    var soundVolume: Float = 0.4f

    /** code: 0 tick, 1 click, 2 heavy click (enter, delete word), 3 long press */
    fun performHaptic(code: Int) {
        if (soundEnabled) playSound(code)
        if (!isEnabled || vibrator == null || !vibrator.hasVibrator()) return

        try {
            val duration = when (code) {
                0 -> 8L
                1 -> 15L
                2 -> 25L
                3 -> 40L
                else -> 12L
            }
            val baseAmplitude = when (code) {
                0 -> 60
                1 -> 120
                2 -> 200
                3 -> 255
                else -> 100
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && vibrator.hasAmplitudeControl()) {
                val amplitude = (baseAmplitude * strength / 60).coerceIn(1, 255)
                vibrator.vibrate(VibrationEffect.createOneShot(duration, amplitude))
            } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                // No amplitude control: weak settings use the lightest predefined effect
                val effect = when {
                    strength < 35 || code == 0 -> VibrationEffect.EFFECT_TICK
                    code == 2 || code == 3 -> VibrationEffect.EFFECT_HEAVY_CLICK
                    else -> VibrationEffect.EFFECT_CLICK
                }
                vibrator.vibrate(VibrationEffect.createPredefined(effect))
            } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                vibrator.vibrate(VibrationEffect.createOneShot(duration * strength / 60, VibrationEffect.DEFAULT_AMPLITUDE))
            } else {
                @Suppress("DEPRECATION")
                vibrator.vibrate((12L * strength / 60).coerceAtLeast(3))
            }
        } catch (_: Exception) {
            // Ignore system vibration errors
        }
    }

    private fun playSound(code: Int) {
        val effect = if (code == 2) AudioManager.FX_KEYPRESS_RETURN else AudioManager.FX_KEYPRESS_STANDARD
        try {
            audio?.playSoundEffect(effect, soundVolume)
        } catch (_: Exception) {
            // Sound effects are best effort
        }
    }
}
