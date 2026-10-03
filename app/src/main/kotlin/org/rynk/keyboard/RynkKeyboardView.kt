package org.rynk.keyboard

import android.content.Context
import android.content.res.Configuration
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Typeface
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.util.AttributeSet
import android.view.MotionEvent
import android.view.View
import android.view.WindowInsets

class RynkKeyboardView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0
) : View(context, attrs, defStyleAttr) {

    private var frontBitmap: Bitmap? = null
    private var backBitmap: Bitmap? = null
    private var onEventsReadyListener: (() -> Unit)? = null
    private var isInitialized = false
    private var bottomInset: Int = 0

    private val repeatHandler = Handler(Looper.getMainLooper())
    private var isBackspaceRepeating = false
    private var backspaceDownTime = 0L

    private val backspaceRepeatRunnable = object : Runnable {
        override fun run() {
            if (!NativeBridge.isLibraryLoaded()) return
            isBackspaceRepeating = true
            val heldDuration = SystemClock.uptimeMillis() - backspaceDownTime

            // Accelerate deletion as hold time increases
            val (count, delayMs) = when {
                heldDuration > 2500L -> 2 to 35L  // Fast deletion (2 chars every 35ms)
                heldDuration > 1200L -> 1 to 45L  // Medium acceleration (1 char every 45ms)
                else -> 1 to 60L                  // Initial repeat rate (1 char every 60ms)
            }

            NativeBridge.nativeRepeatBackspace(count)
            onEventsReadyListener?.invoke()
            invalidate()

            repeatHandler.postDelayed(this, delayMs)
        }
    }

    var currentThemeId: Int = 1
        private set

    // Pre-allocated text rendering paints for maximum 60/120fps performance
    private val textPaintRegular = Paint(Paint.ANTI_ALIAS_FLAG or Paint.SUBPIXEL_TEXT_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.create("sans-serif", Typeface.NORMAL)
    }

    private val textPaintMedium = Paint(Paint.ANTI_ALIAS_FLAG or Paint.SUBPIXEL_TEXT_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
    }

    private val textPaintBold = Paint(Paint.ANTI_ALIAS_FLAG or Paint.SUBPIXEL_TEXT_FLAG).apply {
        textAlign = Paint.Align.CENTER
        typeface = Typeface.create("sans-serif", Typeface.BOLD)
    }

    fun setOnEventsReadyListener(listener: () -> Unit) {
        this.onEventsReadyListener = listener
    }

    fun getThemeBgColor(themeId: Int): Int {
        return when (themeId) {
            1 -> Color.rgb(238, 240, 245) // Light theme #EEF0F5
            2 -> Color.BLACK              // AMOLED
            3 -> Color.rgb(30, 22, 42)    // Sunset
            else -> Color.rgb(56, 58, 64) // Soft Yandex Graphite #383A40
        }
    }

    override fun onApplyWindowInsets(insets: WindowInsets): WindowInsets {
        val navInset = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            insets.getInsets(WindowInsets.Type.navigationBars()).bottom
        } else {
            @Suppress("DEPRECATION")
            insets.systemWindowInsetBottom
        }
        if (bottomInset != navInset) {
            bottomInset = navInset
            requestLayout()
        }
        return super.onApplyWindowInsets(insets)
    }

    private var isEmojiMode: Boolean = false

    fun checkModeChange() {
        if (!NativeBridge.isLibraryLoaded()) return
        val mode = NativeBridge.nativeGetMode()
        val emojiActive = (mode == 3)
        if (isEmojiMode != emojiActive) {
            isEmojiMode = emojiActive
            requestLayout()
        }
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val width = MeasureSpec.getSize(widthMeasureSpec)
        val density = resources.displayMetrics.density
        val isLandscape = resources.configuration.orientation == Configuration.ORIENTATION_LANDSCAPE

        val desiredHeightDp = if (isLandscape) 200f else if (isEmojiMode) 350f else 280f
        val contentHeight = (desiredHeightDp * density).toInt()
        val totalHeight = contentHeight + bottomInset

        setMeasuredDimension(width, totalHeight)
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        if (w <= 0 || h <= 0) return

        if (!NativeBridge.isLibraryLoaded()) return

        val contentH = (h - bottomInset).coerceAtLeast(1)

        if (frontBitmap == null || frontBitmap?.width != w || frontBitmap?.height != contentH) {
            frontBitmap?.recycle()
            backBitmap?.recycle()
            frontBitmap = Bitmap.createBitmap(w, contentH, Bitmap.Config.ARGB_8888)
            backBitmap = Bitmap.createBitmap(w, contentH, Bitmap.Config.ARGB_8888)
        }

        val density = resources.displayMetrics.density
        val prefs = context.getSharedPreferences("rynk_prefs", Context.MODE_PRIVATE)
        val themeId = prefs.getInt("theme_id", 1)
        currentThemeId = themeId
        setBackgroundColor(getThemeBgColor(themeId))

        if (!isInitialized) {
            NativeBridge.nativeInit(w.toFloat(), contentH.toFloat(), density, themeId)
            UserDictionaryManager(context).syncToNative()
            isInitialized = true
        } else {
            NativeBridge.nativeResize(w.toFloat(), contentH.toFloat(), density)
        }
    }

    private val longPressHandler = Handler(Looper.getMainLooper())
    private var longPressStartX = 0f
    private var longPressStartY = 0f
    private var isLongPressTriggered = false

    private val longPressRunnable = Runnable {
        if (!NativeBridge.isLibraryLoaded()) return@Runnable
        isLongPressTriggered = true
        val now = SystemClock.uptimeMillis()

        val dp = resources.displayMetrics.density
        val suggestionBarH = 44f * dp
        if (longPressStartY < suggestionBarH) {
            handleSuggestionBarLongPress(longPressStartX, longPressStartY)
            return@Runnable
        }

        NativeBridge.nativeTick(now)
        onEventsReadyListener?.invoke()
        invalidate()
    }

    private fun handleSuggestionBarLongPress(x: Float, y: Float) {
        val word = NativeBridge.nativeGetSuggestionAt(x, y)
        if (word.isEmpty() || word.contains("...") || SvgIcons.isIcon(word)) {
            return
        }

        (context as? RynkInputMethodService)?.showRemoveWordDialog(word)
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (!NativeBridge.isLibraryLoaded()) return super.onTouchEvent(event)

        val contentH = (height - bottomInset).toFloat()
        val actionMasked = event.actionMasked
        val timeMs = event.eventTime

        // Multitouch: Iterate over all active pointers on ACTION_MOVE
        if (actionMasked == MotionEvent.ACTION_MOVE) {
            var anyHandled = false
            for (i in 0 until event.pointerCount) {
                val pid = event.getPointerId(i)
                val px = event.getX(i)
                val py = event.getY(i)
                if (py > contentH) continue

                val distSq = (px - longPressStartX) * (px - longPressStartX) + (py - longPressStartY) * (py - longPressStartY)
                if (distSq > 500f && !isLongPressTriggered) {
                    longPressHandler.removeCallbacks(longPressRunnable)
                }
                if (px < longPressStartX - 25f) {
                    repeatHandler.removeCallbacks(backspaceRepeatRunnable)
                }

                val h = NativeBridge.nativeOnTouchEvent(2 /* Move */, pid, px, py, timeMs)
                if (h) anyHandled = true
            }
            checkModeChange()
            onEventsReadyListener?.invoke()
            invalidate()
            return anyHandled || true
        }

        // Multitouch: Handle ACTION_CANCEL cleanly for all active touches
        if (actionMasked == MotionEvent.ACTION_CANCEL) {
            repeatHandler.removeCallbacks(backspaceRepeatRunnable)
            longPressHandler.removeCallbacks(longPressRunnable)
            isBackspaceRepeating = false
            isLongPressTriggered = false
            for (i in 0 until event.pointerCount) {
                val pid = event.getPointerId(i)
                val px = event.getX(i)
                val py = event.getY(i)
                NativeBridge.nativeOnTouchEvent(3 /* Cancel */, pid, px, py, timeMs)
            }
            checkModeChange()
            onEventsReadyListener?.invoke()
            invalidate()
            return true
        }

        val actionIndex = event.actionIndex
        val pointerId = event.getPointerId(actionIndex)
        val x = event.getX(actionIndex)
        val y = event.getY(actionIndex)

        if (y > contentH) {
            repeatHandler.removeCallbacks(backspaceRepeatRunnable)
            longPressHandler.removeCallbacks(longPressRunnable)
            return false
        }

        // Handle continuous backspace hold and repeat
        if (actionMasked == MotionEvent.ACTION_DOWN || actionMasked == MotionEvent.ACTION_POINTER_DOWN) {
            longPressStartX = x
            longPressStartY = y
            isLongPressTriggered = false
            longPressHandler.removeCallbacks(longPressRunnable)
            longPressHandler.postDelayed(longPressRunnable, 350L)

            if (NativeBridge.nativeIsBackspaceAt(x, y)) {
                isBackspaceRepeating = false
                backspaceDownTime = SystemClock.uptimeMillis()
                repeatHandler.removeCallbacks(backspaceRepeatRunnable)
                repeatHandler.postDelayed(backspaceRepeatRunnable, 350L)
            }
        } else if (actionMasked == MotionEvent.ACTION_UP || actionMasked == MotionEvent.ACTION_POINTER_UP) {
            repeatHandler.removeCallbacks(backspaceRepeatRunnable)
            longPressHandler.removeCallbacks(longPressRunnable)

            if (isBackspaceRepeating) {
                isBackspaceRepeating = false
                isLongPressTriggered = false
                // Suppress extra release delete after repeating
                NativeBridge.nativeOnTouchEvent(3 /* Cancel */, pointerId, x, y, timeMs)
                invalidate()
                return true
            }

            if (isLongPressTriggered) {
                isLongPressTriggered = false
                // Proceed with normal ACTION_UP so Rust commits the selected alternate key
            }
        }

        val rustAction = when (actionMasked) {
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> 0
            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP -> 1
            else -> return super.onTouchEvent(event)
        }

        val handled = NativeBridge.nativeOnTouchEvent(rustAction, pointerId, x, y, timeMs)

        checkModeChange()

        // Poll keyboard output events for the InputMethodService
        onEventsReadyListener?.invoke()

        invalidate()
        return handled
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)

        // 1. Fill entire canvas (including navigation bar area) with theme background
        val bgColor = getThemeBgColor(currentThemeId)
        canvas.drawColor(bgColor)

        val targetBmp = backBitmap ?: return
        if (!NativeBridge.isLibraryLoaded()) return

        // 2. Double-buffered Rust rendering: render into backBitmap then swap
        val nowMs = SystemClock.uptimeMillis()
        val hasActiveAnimation = NativeBridge.nativeRender(targetBmp, nowMs)

        val temp = frontBitmap
        frontBitmap = backBitmap
        backBitmap = temp

        val readyBmp = frontBitmap ?: return
        canvas.drawBitmap(readyBmp, 0f, 0f, null)

        // 3. Draw text labels on top using native Android system font (including color emojis)
        drawSystemTextLabels(canvas)

        if (hasActiveAnimation) {
            postInvalidateOnAnimation()
        }
    }

    private var cachedLabelsVersion: Long = -1L
    private var cachedLabels: List<TextLabelItem> = emptyList()

    private class TextLabelItem(
        val text: String,
        val cx: Float,
        val cy: Float,
        val fontSize: Float,
        val color: Int,
        val isBold: Boolean,
        val labelType: Int
    )

    private fun drawSystemTextLabels(canvas: Canvas) {
        val version = NativeBridge.nativeGetLabelsVersion()
        if (version != cachedLabelsVersion) {
            val labelsString = NativeBridge.nativeGetTextLabels()
            if (labelsString.isNotEmpty()) {
                val lines = labelsString.split("\n")
                val list = ArrayList<TextLabelItem>(lines.size)
                for (line in lines) {
                    val trimmed = line.trim()
                    if (trimmed.isEmpty()) continue

                    val parts = trimmed.split("\t")
                    if (parts.size < 10) continue

                    val text = parts[0]
                    val cx = parts[1].toFloatOrNull() ?: continue
                    val cy = parts[2].toFloatOrNull() ?: continue
                    val fontSize = parts[3].toFloatOrNull() ?: continue
                    val r = parts[4].toIntOrNull() ?: continue
                    val g = parts[5].toIntOrNull() ?: continue
                    val b = parts[6].toIntOrNull() ?: continue
                    val a = parts[7].toIntOrNull() ?: continue
                    val isBold = parts[8] == "1"
                    val labelType = parts[9].toIntOrNull() ?: 0

                    list.add(
                        TextLabelItem(
                            text = text,
                            cx = cx,
                            cy = cy,
                            fontSize = fontSize,
                            color = Color.argb(a, r, g, b),
                            isBold = isBold,
                            labelType = labelType
                        )
                    )
                }
                cachedLabels = list
            } else {
                cachedLabels = emptyList()
            }
            cachedLabelsVersion = version
        }

        val dp = resources.displayMetrics.density
        val tab_bar_h = 40f * dp
        val bottom_bar_h = 40f * dp
        val elevation_h = 36f * dp
        val bot_y = (height - bottomInset).toFloat() - elevation_h - bottom_bar_h

        for (item in cachedLabels) {
            // Clip emoji labels: in normal emoji mode they must be in grid area (below tab bar, above bottom bar)
            // In search mode the strip emojis (labelType==6) can be in header area, so we only clip
            // those below the keyboard content area (navigation zone)
            if (item.labelType == 6 && item.cy > bot_y) {
                continue
            }

            // Check if this is an SVG vector icon (Shift, Backspace, Enter, Globe, Emoji, Settings, Back)
            if (item.labelType != 6 && (item.labelType == 5 || SvgIcons.isIcon(item.text))) {
                val iconSize = (item.fontSize * 1.25f).coerceAtLeast(18f)
                val drawn = SvgIcons.draw(canvas, item.text, item.cx, item.cy, iconSize, item.color)
                if (drawn) {
                    continue
                }
            }

            val paint = when {
                item.isBold -> textPaintBold
                item.labelType == 0 -> textPaintRegular   // Character keys
                item.labelType == 6 -> textPaintRegular   // Grid emojis
                item.labelType == 1 -> textPaintRegular   // Secondary sub-labels
                item.labelType == 2 -> textPaintMedium    // Suggestion chips & shortcuts
                item.labelType == 3 -> textPaintBold      // Popups
                item.labelType == 4 -> textPaintMedium    // Brand / Space language
                else -> textPaintRegular
            }

            paint.textSize = item.fontSize
            paint.color = item.color

            val fontMetrics = paint.fontMetrics
            val baselineY = item.cy - (fontMetrics.ascent + fontMetrics.descent) / 2f

            canvas.drawText(item.text, item.cx, baselineY, paint)
        }
    }

    fun applyTheme(themeId: Int) {
        currentThemeId = themeId
        setBackgroundColor(getThemeBgColor(themeId))
        cachedLabelsVersion = -1L
        if (NativeBridge.isLibraryLoaded()) {
            NativeBridge.nativeSetTheme(themeId)
            invalidate()
        }
    }

    fun applyLanguage(langId: Int) {
        cachedLabelsVersion = -1L
        if (NativeBridge.isLibraryLoaded()) {
            NativeBridge.nativeSetLanguage(langId)
            invalidate()
        }
    }

    fun resetState() {
        cachedLabelsVersion = -1L
        if (NativeBridge.isLibraryLoaded()) {
            NativeBridge.nativeReset()
            checkModeChange()
            invalidate()
        }
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        repeatHandler.removeCallbacks(backspaceRepeatRunnable)
        longPressHandler.removeCallbacks(longPressRunnable)
        frontBitmap?.recycle()
        frontBitmap = null
        backBitmap?.recycle()
        backBitmap = null
        if (isInitialized && NativeBridge.isLibraryLoaded()) {
            NativeBridge.nativeDestroy()
            isInitialized = false
        }
    }
}
