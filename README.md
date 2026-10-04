<div align="center">

# ⚡ Rynk Keyboard

**Высокопроизводительная Android-клавиатура нового поколения на Rust и Kotlin**  
*Next-generation, ultra-responsive Android keyboard powered by Rust and Kotlin*

[![Platform](https://img.shields.io/badge/Platform-Android%207.0%2B%20(API%2024--35)-brightgreen?style=for-the-badge&logo=android)](https://developer.android.com)
[![Core](https://img.shields.io/badge/Core-Rust%202026-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Release](https://img.shields.io/badge/Download-Release%20APK%20(v1.0.0)-success?style=for-the-badge&logo=android)](https://github.com/Kolya62/Rynk-Keyboard/releases/latest)
[![Rendering](https://img.shields.io/badge/Rendering-Direct%20SDF%20Pixel%20Buffer-blue?style=for-the-badge)](https://github.com)
[![Performance](https://img.shields.io/badge/Performance-120%20FPS%20%7C%20%3C1ms%20Latency-purple?style=for-the-badge)](https://github.com)
[![License](https://img.shields.io/badge/License-MPL%202.0-blue?style=for-the-badge)](LICENSE)

[🇷🇺 Русский](#-на-русском) • [🇬🇧 English](#-in-english)

---

</div>

<a name="-на-русском"></a>
## 🇷🇺 На русском

### 🌟 О проекте

**Rynk Keyboard** — это современная высокопроизводительная системная клавиатура для Android, объединяющая бескомпромиссную скорость системного языка **Rust** с нативной интеграцией **Android SDK (Kotlin)**.

В отличие от традиционных клавиатур, где рендеринг завязан на тяжеловесный стек Java/Kotlin UI или веб-компоненты, Rynk производит всю математику, обработку мультитач-жестов, физику анимаций, предиктивный поиск и 2D-растеризацию интерфейса **непосредственно в нативном коде Rust** с прямой записью пикселей в аппаратный буфер Android через `AndroidBitmap_lockPixels` (`libjnigraphics.so`).

---

### 🚀 Ключевые возможности

1. **Мгновенный отклик Touch-to-Pixel (< 1 мс)**:
   - Растеризатор на базе знаковых функций расстояния (**Signed Distance Fields / SDF**) с субпиксельным сглаживанием (anti-aliasing).
   - Мягкие объёмные тени под клавишами и эффект тактильной глубины в реальном времени.
   - Физические микро-анимации нажатий, всплывающие превью (Popups) и ripple-эффекты с частотой обновления до 120 FPS.
2. **Интеллектуальная предиктивная система и автозамена**:
   - Префиксное дерево **Trie** с временем поиска < 0.5 мс.
   - Продвинутая автозамена с учётом соседних клавиш, опечаток, перестановок и диакритики.
   - Контекстное предсказание следующего слова по биграммам для всех поддерживаемых языков.
   - Канонический регистр аббревиатур (`OST`, `macOS`, `СПб`, `СНиП`, `хз`, `спс`).
   - Мгновенная отмена автозамены: нажатие Backspace сразу после пробела восстанавливает введённое слово и защищает его от повторной автокоррекции.
   - Защита словаря: опечатки не засоряют пользовательский лексикон.
   - Глубокая морфология: контекстный анализ приставок, суффиксов, окончаний и предлогов, а также современные слова, сокращения и сленг.
3. **Глобальная мультиязычность (85+ языков)**:
   - **Китайские языки**: Упрощённый китайский (中文 简体), Традиционный китайский (中文 繁體), Кантонский диалект (粵語 香港).
   - **Корейский язык**: Нативная двухраскладочная клавиатура 2-Set Hangul (두벌식) с автоматической сборкой слогов и поддержкой сдвоенных согласных.
   - **Японский язык**: Японский ввод Romaji (日本語) с частотным словарем и автозаменой.
   - **Вьетнамский язык**: Вьетнамский ввод (Tiếng Việt) с полной поддержкой тонов и диакритических знаков.
   - **Европейские и мировые языки**: Русский, Английский, Арабский, Польский, Чешский, Румынский, Немецкий, Французский, Испанский, Итальянский, Турецкий, Украинский, Казахский, Иврит, Хинди и многие другие (более 85 языков мира).
   - Быстрое переключение раскладок свайпом влево/вправо по пробелу или клавишей глобуса.
4. **Сенсорные зоны и жесты**:
   - **Бесшовный сенсорный слой без мёртвых зон (Gapless Hit-Boxes)**: активные сенсорные области клавиш расширены и перекрывают зазоры между кнопками, исключая промахи при быстром слепом наборе.
   - **Символ подчёркивания `_`**: доступен на цифро-символьной раскладке и по удержанию дефиса/пробела.
   - **Свайп по пробелу**: быстрое переключение языка раскладки свайпом влево или вправо (без сдвига курсора).
   - **Свайп влево от Backspace**: мгновенное удаление целого слова.
   - **Long-Press (удержание)**: всплывающее меню акцентов, цифр и спецсимволов.
   - Двойной тап по Shift для CapsLock с визуальным индикатором.
5. **Умный буфер обмена и адаптация под экраны**:
   - **Однократная подсказка буфера обмена**: превью скопированного текста отображается ровно один раз, исчезает сразу после вставки; доступна кнопка быстрого удаления текста из буфера.
   - **Оптимизация для планшетов и смартфонов**: специализированные адаптивные профили разметки (`sw600dp`) и альбомной ориентации для максимального удобства набора на планшетах.
6. **Все эмодзи Unicode 16.0 со встроенным поиском**:
   - Полная база из 3 790 эмодзи Unicode 16.0 с разделением по категориям.
   - Быстрый встроенный поиск эмодзи на русском и английском языках с ключевыми словами CLDR.
   - Текстовые шорткаты (`:)` $\rightarrow$ 😊, `<3` $\rightarrow$ ❤️).
7. **Темы оформления**:
   - **Rynk Dark (Cyan)** — фирменный графитовый стиль с неоновым акцентом.
   - **Rynk Light (Sapphire)** — чистая жемчужная тема с сапфировым акцентом.
   - **AMOLED Black** — абсолютный чёрный (`#000000`) для экономии батареи на OLED-экранах.
   - **Sunset Twilight** — тёплая вечерняя палитра.
8. **Конфиденциальность и безопасность (Privacy-First)**:
   - Никаких сетевых разрешений (`INTERNET` отсутствует в AndroidManifest.xml).
   - Интеллектуальное распознавание типов полей ввода (`EditorInfo`): подсказки, автозамена, локальное обучение и превью буфера обмена автоматически блокируются в полях ввода паролей.
   - Локальные словари защищены правилами резервного копирования (`backup_rules.xml`, `data_extraction_rules.xml`) и никогда не отправляются в облако.
9. **Высокоскоростной бинарный JNI-протокол и двойная буферизация**:
   - Двунаправленный обмен событиями через бинарный формат `[count: u32 LE] [type: u8][len: u32 LE][payload: N]`, исключающий строковые аллокации.
   - Двойная буферизация кадров (`frontBitmap` / `backBitmap`) с прямым замком пикселей в Rust.

---

### 🏗️ Архитектура

Архитектура **Rynk Keyboard** построена на строгом принципе разделения ответственности: **100% вычислительной логики, геометрии и рендеринга сосредоточено в Rust Core**, а Android SDK (Kotlin) используется исключительно в роли ультратонкого системного моста ввода без тяжелых иерархий `View`:

```
Rynk/
├── rust-core/                        # ВЫСОКОПРОИЗВОДИТЕЛЬНОЕ ЯДРО (RUST 2024 / NIGHTLY-COMPATIBLE)
│   ├── Cargo.toml                    # cdylib + rlib конфигурация, LTO, opt-level = 3
│   └── src/
│       ├── lib.rs                    # Модуль интеграции ядра, стресс-тесты и замеры задержки
│       ├── jni_bridge.rs             # Zero-copy JNI FFI интерфейс через прямой бинарный ByteBuffer
│       ├── keyboard/
│       │   ├── mod.rs                # Главный координатор KeyboardEngine (состояния, события, пайплайн)
│       │   ├── key.rs                # Геометрия клавиш, бесшовные расширенные хит-боксы без слепых зон
│       │   ├── layout.rs             # Генерация матриц раскладок (85+ языков, планшетные и телефонные сетки)
│       │   ├── state.rs              # Состояние ввода (Shift/Caps, режим, язык, очередь бинарных событий)
│       │   ├── touch.rs              # Мультитач-трекер, лонг-пресс, свайп пробела (переключение языка)
│       │   └── hangul.rs             # 2-Set Hangul композитор (автоматическая сборка/разборка корейских слогов)
│       ├── render/
│       │   ├── mod.rs                # Координатор конвейера отрисовки (двойная буферизация кадров)
│       │   ├── canvas.rs             # 2D SDF-растеризатор пикселей, субпиксельный AA, тени клавиш
│       │   ├── font.rs               # Рендерер векторных глифов и системных иконок
│       │   ├── popup.rs              # Всплывающие превью и контекстные меню символов
│       │   ├── animation.rs          # Плавная интерполяция нажатий и ripple-эффектов (до 120 FPS)
│       │   └── theme.rs              # Цветовые темы (Rynk Dark, Rynk Light, AMOLED Black, Sunset Twilight)
│       ├── prediction/
│       │   ├── mod.rs                # Координатор предиктивного ввода, автокоррекции и словарей
│       │   ├── trie.rs               # Префиксное дерево Trie для сверхбыстрого поиска (< 0.5 мс)
│       │   ├── dictionary.rs         # Частотные словари 85+ языков, атомарный пользовательский лексикон
│       │   ├── autocorrect.rs        # Ранжирование по Дамерау-Левенштейну с учётом геометрии клавиатуры
│       │   ├── typos.rs              # Таблицы частых орфографических опечаток и соседних клавиш
│       │   ├── morphology.rs         # Морфологический анализ (приставки, суффиксы, окончания, предлоги)
│       │   ├── cjk.rs                # CJK-модуль: Pinyin (китайский) и Romaji (японский) кандидаты
│       │   ├── suggestions.rs        # Унифицированное ранжирование кандидатов подсказок
│       │   └── data/                 # Базы слов на 85+ языках, биграммы, сленг, сокращения и лексика
│       └── emoji/
│           ├── mod.rs                # Менеджер каталога и рендера эмодзи (Unicode 16.0)
│           ├── data.rs               # База данных из 3 790 эмодзи со всеми категориями
│           └── search.rs             # Быстрый многоязычный поиск эмодзи по ключевым словам CLDR
└── app/                              # МИНИМАЛЬНЫЙ СИСТЕМНЫЙ СЛОЙ ANDROID SDK (KOTLIN)
    ├── build.gradle.kts              # Автоматизация кросс-компиляции NDK + AGP + Proguard/R8
    └── src/main/
        ├── AndroidManifest.xml       # Регистрация InputMethodService (0 сетевых разрешений)
        ├── kotlin/org/rynk/keyboard/
        │   ├── NativeBridge.kt       # JNI-загрузчик librynk_core.so и бинарная десериализация
        │   ├── RynkKeyboardView.kt   # Hardware-accelerated View с Bitmap буфером и адаптацией под экраны
        │   ├── RynkInputMethodService.kt # Системный IME сервис: жизненный цикл, буфер обмена, безопасность
        │   ├── HapticManager.kt      # Тактильная отдача с низкой задержкой (VibrationEffect)
        │   ├── UserDictionaryManager.kt # Синхронизация и локальное хранилище пользовательских слов
        │   ├── SetupWizardActivity.kt# Мастер первоначальной настройки с окном поддержки автора
        │   ├── SettingsActivity.kt   # Экран настроек (темы, языки, высота, планшет, поддержка автора)
        │   └── SvgIcons.kt           # Компактные векторные пиктограммы интерфейса
        └── res/                      # Полная локализация на 85+ языков и планшетные ресурсы (sw600dp)
```

---

### 🛠️ Сборка из исходного кода

#### Требования:
- **JDK**: OpenJDK 21 или 27
- **Android SDK & NDK**: NDK версия `26.3.11579264`
- **Rust Toolchain**: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`

#### Команды сборки:
```bash
# 1. Прогон всех unit-тестов и бенчмарков Rust ядра (31 тест)
cargo test --manifest-path rust-core/Cargo.toml

# 2. Прогон модульных тестов Android
export JAVA_HOME=/usr/lib/jvm/java-27-openjdk
./gradlew test

# 3. Компиляция Rust библиотек и сборка единого Release APK (с R8 и сжатием ресурсов)
./gradlew buildRustCore assembleRelease

# Готовый подписанный релизный APK:
# app/build/outputs/apk/release/app-release.apk
```

---

<a name="-in-english"></a>
## 🇬🇧 In English

### 🌟 Overview

**Rynk Keyboard** is a state-of-the-art, high-performance Android input method engineered for extreme responsiveness by combining **Rust** for the entire core processing and rendering with **Kotlin** strictly for mandatory Android SDK input method service bindings.

Unlike standard mobile keyboards burdened by heavy Java/Kotlin UI hierarchies or web runtimes, Rynk executes all layout math, multi-touch gesture processing, animation timing, predictive search, and 2D pixel rendering **directly in native Rust**. The resulting frame is flushed directly to the hardware frame buffer via `AndroidBitmap_lockPixels` (`libjnigraphics.so`) with zero memory copies.

---

### 🚀 Highlights & Features

1. **Zero-Latency Touch-to-Pixel (< 1 ms)**:
   - High-performance Signed Distance Fields (**SDF**) 2D rasterizer with subpixel anti-aliasing.
   - Smooth elevation drop shadows under keys for tactile realism.
   - Key-press micro-animations, popups, and ripple effects rendering up to 120 FPS.
2. **Next-Gen Autocorrect & Morphological Engine**:
   - In-memory **Trie** structure yielding lookups under 0.5 ms.
   - Proximity-aware autocorrect accounting for keyboard geometry, transpositions, and diacritics.
   - Cross-language bigram next-word prediction.
   - Canonical abbreviation casing preservation (`OST`, `macOS`, `AFK`).
   - Instant autocorrect undo: tapping Backspace once after spacebar restores the exact user input and prevents re-correction.
   - Deep morphological analysis: context-aware handling of prefixes, suffixes, inflectional endings, and prepositions.
   - Clean user dictionary guarantee: uncorrected typos never pollute the dictionary.
3. **85+ Supported Languages**:
   - **Chinese**: Simplified Chinese (中文 简体), Traditional Chinese (中文 繁體), Cantonese (粵語 香港).
   - **Korean**: Native 2-Set Hangul (두벌식) layout with syllable composition and double-consonant shift support.
   - **Japanese**: Romaji Japanese (日本語) with dedicated vocabulary and suggestion engine.
   - **Vietnamese**: Vietnamese (Tiếng Việt) with full diacritic and tone mark handling.
   - **Global & European languages**: English, Russian, Arabic, Polish, Czech, Romanian, German, French, Spanish, Italian, Hebrew, Hindi, and 70+ more.
   - Instant language switching via spacebar swipe or dedicated globe key.
4. **Touch Precision & Gestures**:
   - **Gapless Hit-Box Layer**: Active touch hit-boxes dynamically bridge inter-key spacing, eliminating dead zones and accidental mis-taps.
   - **Dedicated Underscore `_` Symbol**: Instantly accessible on numeric/symbol layouts and via long-press.
   - **Spacebar Swipe Language Switch**: Swiftly switch keyboard language layout by swiping left or right across the spacebar (without moving the text cursor).
   - **Backspace Swipe**: Swipe left from Backspace to delete whole words in one stroke.
   - **Long-Press Diacritics**: Hold any key to reveal alternative characters, symbols, and digits.
   - Double-tap Shift for persistent CapsLock with visual state indicator.
5. **Smart Clipboard & Tablet Optimization**:
   - **One-Time Clipboard Suggestion**: Copied snippets appear as a single-use chip, disappearing automatically upon insertion, with one-tap clipboard deletion.
   - **Adaptive Tablet Support**: Tailored multi-column key matrices and landscape geometry for large screens (`sw600dp`).
6. **Full Unicode 16.0 Emoji Catalog & Search**:
   - Complete database of 3,790 Unicode 16.0 emojis across all categories.
   - High-speed interactive search in Russian and English based on CLDR keywords.
   - Instant inline emoji shortcuts (`:)` $\rightarrow$ 😊, `<3` $\rightarrow$ ❤️).
7. **Curated Themes**:
   - **Rynk Dark (Cyan)** — Signature graphite background with neon cyan accents.
   - **Rynk Light (Sapphire)** — Crisp pearl palette with deep sapphire highlights.
   - **AMOLED Black** — Pure `#000000` dark theme for maximum OLED battery savings.
   - **Sunset Twilight** — Warm gradient evening palette.
8. **100% Privacy by Design**:
   - Zero network permissions (`INTERNET` is completely absent from `AndroidManifest.xml`).
   - Sensitive field intelligence (`EditorInfo`): suggestions, autocorrect, n-gram learning, and clipboard preview are strictly suppressed when editing password fields.
   - User dictionaries and adaptive learning history are excluded from cloud backups via `backup_rules.xml` and `data_extraction_rules.xml`.
9. **High-Throughput Binary Protocol & Double Buffering**:
   - Length-prefixed binary event protocol `[count: u32 LE] [type: u8][len: u32 LE][payload: N]` eliminating string allocations during typing.
   - Double-buffered frame presentation (`frontBitmap` / `backBitmap`) delivering consistent 120 FPS frame rates.

---

### 🏗️ Architecture

Rynk Keyboard strictly enforces clean separation of concerns: **100% of calculation, layout geometry, predictive search, and rendering resides in the native Rust Core**, while the Android SDK (Kotlin) serves solely as a minimal system input bridge without heavy `View` hierarchies:

```
Rynk/
├── rust-core/                        # HIGH-PERFORMANCE CORE (RUST 2024 / NIGHTLY-COMPATIBLE)
│   ├── Cargo.toml                    # cdylib + rlib configuration, LTO, opt-level = 3
│   └── src/
│       ├── lib.rs                    # Core integration entry point, stress tests & latency benchmarks
│       ├── jni_bridge.rs             # Zero-copy JNI FFI interface over direct binary ByteBuffer
│       ├── keyboard/
│       │   ├── mod.rs                # Central KeyboardEngine coordinator (state, events, pipeline)
│       │   ├── key.rs                # Key geometry, gapless touch hit-box expansion
│       │   ├── layout.rs             # Layout matrix generator (85+ languages, tablet & phone grids)
│       │   ├── state.rs              # Runtime input state (Shift/Caps, mode, language, binary output queue)
│       │   ├── touch.rs              # Multitouch tracker, long-press timer, spacebar swipe language switch
│       │   └── hangul.rs             # 2-Set Hangul composer (syllable assembly & jamo decomposition)
│       ├── render/
│       │   ├── mod.rs                # Frame rendering coordinator (double-buffered frame presentation)
│       │   ├── canvas.rs             # 2D SDF pixel rasterizer, subpixel AA, soft key elevation shadows
│       │   ├── font.rs               # Vector glyph & system icon renderer
│       │   ├── popup.rs              # Key preview bubbles & diacritic popup selector
│       │   ├── animation.rs          # Key-press micro-animations & ripple effect interpolation (up to 120 FPS)
│       │   └── theme.rs              # Color themes (Rynk Dark, Rynk Light, AMOLED Black, Sunset Twilight)
│       ├── prediction/
│       │   ├── mod.rs                # Predictive typing coordinator, autocorrect & lexicon engine
│       │   ├── trie.rs               # High-speed prefix Trie data structure (< 0.5 ms lookup)
│       │   ├── dictionary.rs         # 85+ language dictionaries, atomic persistent user lexicon
│       │   ├── autocorrect.rs        # Damerau-Levenshtein ranking with keyboard layout proximity
│       │   ├── typos.rs              # Common spelling typos & neighboring key substitution maps
│       │   ├── morphology.rs         # Morphological analysis (prefixes, suffixes, endings, prepositions)
│       │   ├── cjk.rs                # CJK engine: Pinyin (Chinese) and Romaji (Japanese) candidates
│       │   ├── suggestions.rs        # Unified candidate scoring & suggestion ranking
│       │   └── data/                 # 85+ language word lists, bigrams, slang, and vocabulary
│       └── emoji/
│           ├── mod.rs                # Unicode 16.0 emoji organizer & renderer
│           ├── data.rs               # Comprehensive 3,790+ emoji database
│           └── search.rs             # Multilingual CLDR keyword-based emoji search engine
└── app/                              # MINIMAL ANDROID SDK BINDINGS (KOTLIN)
    ├── build.gradle.kts              # Automated NDK cross-compilation + AGP + Proguard/R8
    └── src/main/
        ├── AndroidManifest.xml       # InputMethodService declaration (0 network permissions)
        ├── kotlin/org/rynk/keyboard/
        │   ├── NativeBridge.kt       # JNI loader for librynk_core.so & binary deserialization
        │   ├── RynkKeyboardView.kt   # Hardware-accelerated View with double Bitmap buffer
        │   ├── RynkInputMethodService.kt # System IME service: lifecycle, clipboard, private mode security
        │   ├── HapticManager.kt      # Low-latency haptic feedback via VibrationEffect
        │   ├── UserDictionaryManager.kt # Thread-safe persistent storage for user learned words
        │   ├── SetupWizardActivity.kt# Setup wizard with one-time author support dialog
        │   ├── SettingsActivity.kt   # Full settings screen (themes, languages, height, tablet, support)
        │   └── SvgIcons.kt           # Compact vector UI icons
        └── res/                      # Complete localization in 85+ languages & tablet resources (sw600dp)
```

---

### 🛠️ Building from Source

#### Prerequisites:
- **JDK**: OpenJDK 21 or 27
- **Android SDK & NDK**: NDK version `26.3.11579264`
- **Rust Toolchain**: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`

#### Build Instructions:
```bash
# 1. Run full Rust core test suite and benchmarks (31 unit tests)
cargo test --manifest-path rust-core/Cargo.toml

# 2. Run Android unit tests
export JAVA_HOME=/usr/lib/jvm/java-27-openjdk
./gradlew test

# 3. Build native .so libraries and generate single signed Release APK (with R8 and resource shrinking)
./gradlew buildRustCore assembleRelease

# Output signed Release APK:
# app/build/outputs/apk/release/app-release.apk
```

---

### 💖 Поддержать автора / Support the Author

- **Банковская карта / Bank Card**: `4466 1481 2794 9960`
- Доступно разовое окно благодарности при первом запуске, а также постоянный раздел в настройках клавиатуры.
- One-time support dialog shown on first launch, plus a dedicated section in the keyboard settings.
- Спасибо за поддержку разработки проекта Rynk Keyboard! / Thank you for supporting the continuous development of Rynk Keyboard!

---

### 📄 Лицензия / License

- **Русский:** Проект распространяется под свободной лицензией **Mozilla Public License Version 2.0 (MPL-2.0)**. Подробности в файле `LICENSE`.
- **English:** Distributed under the **Mozilla Public License Version 2.0 (MPL-2.0)**. See `LICENSE` for details.

Designed and crafted with passion for performance.

