package org.rynk.keyboard

import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import androidx.core.graphics.PathParser

object SvgIcons {

    private val iconPaths = HashMap<String, Path>()
    private val iconPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.FILL
    }

    init {
        // SVG paths based on authentic Material/Yandex vector icons (24x24 viewBox)
        // 1. Shift Off: crisp upward arrow outline
        register("shift_off", "M12 4.5 L4.5 12 H8.5 V19 H15.5 V12 H19.5 L12 4.5 Z M12 7.33 L16.17 11.5 H13.5 V17 H10.5 V11.5 H7.83 L12 7.33 Z")

        // 2. Shift On: filled bold upward arrow
        register("shift_on", "M12 4.5 L4.5 12 H8.5 V19 H15.5 V12 H19.5 L12 4.5 Z")

        // 3. Caps Lock: filled upward arrow with baseline bar
        register("caps_lock", "M12 3.5 L4.5 11 H8.5 V16 H15.5 V11 H19.5 L12 3.5 Z M7 18 H17 V20.5 H7 V18 Z")

        // 4. Backspace: standard badge with cross
        register("backspace", "M22 3H7c-.69 0-1.23.35-1.59.88L0 12l5.41 8.11c.36.53.9.89 1.59.89h15c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-3 12.59L17.59 17 14 13.41 10.41 17 9 15.59 12.59 12 9 8.41 10.41 7 14 10.59 17.59 7 19 8.41 15.41 12 19 15.59z")

        // 5. Enter: return arrow
        register("enter", "M19 7v4H5.83l3.58-3.59L8 6l-6 6 6 6 1.41-1.41L5.83 13H21V7h-2z")

        // 6. Globe: language switch globe
        register("globe", "M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zm6.93 6h-2.95c-.32-1.25-.78-2.45-1.38-3.56 1.84.63 3.37 1.91 4.33 3.56zM12 4.04c.83 1.2 1.48 2.53 1.91 3.96h-3.82c.43-1.43 1.08-2.76 1.91-3.96zM4.26 14C4.1 13.36 4 12.69 4 12s.1-1.36.26-2h3.38c-.08.66-.14 1.32-.14 2 0 .68.06 1.34.14 2H4.26zm.82 2h2.95c.32 1.25.78 2.45 1.38 3.56-1.84-.63-3.37-1.9-4.33-3.56zm2.95-8H5.08c.96-1.66 2.49-2.93 4.33-3.56C8.81 5.55 8.35 6.75 8.03 8zM12 19.96c-.83-1.2-1.48-2.53-1.91-3.96h3.82c-.43 1.43-1.08 2.76-1.91 3.96zM14.34 14H9.66c-.09-.66-.16-1.32-.16-2 0-.68.07-1.35.16-2h4.68c.09.65.16 1.32.16 2 0 .68-.07 1.34-.16 2zm.25 5.56c.6-1.11 1.06-2.31 1.38-3.56h2.95c-.96 1.65-2.49 2.93-4.33 3.56zM16.36 14c.08-.66.14-1.32.14-2 0-.68-.06-1.34-.14-2h3.38c.16.64.26 1.31.26 2s-.1 1.36-.26 2h-3.38z")

        // 7. Emoji: sentiment smiley
        register("emoji", "M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm3.5-9c.83 0 1.5-.67 1.5-1.5S16.33 8 15.5 8 14 8.67 14 9.5s.67 1.5 1.5 1.5zm-7 0c.83 0 1.5-.67 1.5-1.5S9.33 8 8.5 8 7 8.67 7 9.5 7.67 11 8.5 11zm3.5 6.5c2.33 0 4.31-1.46 5.11-3.5H6.89c.8 2.04 2.78 3.5 5.11 3.5z")

        // 8. Settings: gear icon
        register("settings", "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z")

        // 9. Back arrow
        register("back", "M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z")

        // 10. Enter key editor actions (EditorInfo.IME_ACTION_*)
        register("enter_search", "M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z")
        register("enter_send", "M2.01 21L23 12 2.01 3 2 10l15 2-15 2z")
        register("enter_go", "M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z")
        register("enter_next", "M11.59 7.41L15.17 11H1v2h14.17l-3.59 3.59L13 18l6-6-6-6-1.41 1.41zM20 6v12h2V6h-2z")
        register("enter_done", "M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z")
        iconPaths["back"]?.let { iconPaths["enter_previous"] = it }

        // Direct aliases for Unicode character strings sent from keyboard layout
        iconPaths["shift_off"]?.let { iconPaths["⇧"] = it }
        iconPaths["shift_on"]?.let { iconPaths["⬆"] = it }
        iconPaths["caps_lock"]?.let { iconPaths["⇪"] = it }
        iconPaths["backspace"]?.let { iconPaths["⌫"] = it }
        iconPaths["enter"]?.let { iconPaths["↵"] = it }
        iconPaths["globe"]?.let { iconPaths["🌐"] = it }
        iconPaths["settings"]?.let { iconPaths["⚙"] = it }
        iconPaths["back"]?.let { iconPaths["←"] = it }
        iconPaths["emoji"]?.let { iconPaths["😊"] = it }
    }

    private fun register(name: String, pathData: String) {
        try {
            iconPaths[name] = PathParser.createPathFromPathData(pathData)
        } catch (_: Exception) {}
    }

    fun isIcon(name: String): Boolean = iconPaths.containsKey(name)

    fun draw(canvas: Canvas, iconName: String, cx: Float, cy: Float, targetSize: Float, color: Int): Boolean {
        val path = iconPaths[iconName] ?: return false
        iconPaint.color = color

        val scale = targetSize / 24f
        canvas.save()
        canvas.translate(cx, cy)
        canvas.scale(scale, scale)
        canvas.translate(-12f, -12f)
        canvas.drawPath(path, iconPaint)
        canvas.restore()
        return true
    }
}
