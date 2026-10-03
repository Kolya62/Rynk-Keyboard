package org.rynk.keyboard

import android.content.Context
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager

class HapticManager(context: Context) {

    private val vibrator: Vibrator? = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val manager = context.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as? VibratorManager
        manager?.defaultVibrator
    } else {
        @Suppress("DEPRECATION")
        context.getSystemService(Context.VIBRATOR_SERVICE) as? Vibrator
    }

    var isEnabled: Boolean = true

    fun performHaptic(code: Int) {
        if (!isEnabled || vibrator == null || !vibrator.hasVibrator()) return

        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val effect = when (code) {
                    0 -> VibrationEffect.createPredefined(VibrationEffect.EFFECT_TICK)
                    1 -> VibrationEffect.createPredefined(VibrationEffect.EFFECT_CLICK)
                    2 -> VibrationEffect.createPredefined(VibrationEffect.EFFECT_HEAVY_CLICK)
                    3 -> VibrationEffect.createOneShot(35, 180)
                    else -> VibrationEffect.createPredefined(VibrationEffect.EFFECT_CLICK)
                }
                vibrator.vibrate(effect)
            } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                val duration = when (code) {
                    0 -> 8L
                    1 -> 15L
                    2 -> 25L
                    3 -> 40L
                    else -> 12L
                }
                val amplitude = when (code) {
                    0 -> 60
                    1 -> 120
                    2 -> 200
                    3 -> 255
                    else -> 100
                }
                vibrator.vibrate(VibrationEffect.createOneShot(duration, amplitude))
            } else {
                @Suppress("DEPRECATION")
                vibrator.vibrate(12L)
            }
        } catch (_: Exception) {
            // Ignore system vibration errors
        }
    }
}
