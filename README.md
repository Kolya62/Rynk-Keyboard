<div align="center">

# ⚡ Rynk Keyboard

**Высокопроизводительная Android-клавиатура нового поколения на Rust и Kotlin**  
*Next-generation, ultra-responsive Android keyboard powered by Rust and Kotlin*

[![Platform](https://img.shields.io/badge/Platform-Android%207.0%2B%20(API%2024--35)-brightgreen?style=for-the-badge&logo=android)](https://developer.android.com)
[![Core](https://img.shields.io/badge/Core-Rust%202021-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Rendering](https://img.shields.io/badge/Rendering-Direct%20SDF%20Pixel%20Buffer-blue?style=for-the-badge)](https://github.com)
[![Performance](https://img.shields.io/badge/Performance-120%20FPS%20%7C%20%3C1ms%20Latency-purple?style=for-the-badge)](https://github.com)
[![License](https://img.shields.io/badge/License-MPL%202.0-blue?style=for-the-badge)](LICENSE)

[🇷🇺 Русский](#-на-русском) • [🇬🇧 English](#-in-english)

---

</div>

<a name="-на-русском"></a>
## 🇷🇺 На русском

### 🌟 О проекте

**Rynk Keyboard** — это современная системная клавиатура для Android премиального уровня, объединяющая бескомпромиссную скорость системного языка **Rust** с нативной интеграцией **Android SDK (Kotlin)**.

В отличие от традиционных клавиатур, где рендеринг завязан на тяжеловесный стек Java/Kotlin UI или веб-компоненты, Rynk производит всю математику, обработку мультитач-жестов, физику анимаций, предиктивный поиск и 2D-растеризацию интерфейса **непосредственно в нативном коде Rust** с прямой записью пикселей в аппаратный буфер Android через `AndroidBitmap_lockPixels` (`libjnigraphics.so`).

---

### 🚀 Ключевые возможности

1. **Мгновенный отклик Touch-to-Pixel (< 1 мс)**:
   - Растеризатор на базе знаковых функций расстояния (**Signed Distance Fields / SDF**) с субпиксельным сглаживанием (anti-aliasing).
   - Мягкие объёмные тени под клавишами и эффект тактильной глубины в реальном времени.
   - Физические микро-анимации нажатий, всплывающие превью (Popups) и ripple-эффекты с частотой обновления до 120 FPS.
2. **Интеллектуальная предиктивная система**:
   - Префиксное дерево **Trie** с временем поиска по 50 000+ словам < 0.5 мс.
   - Умная автозамена на основе расстояния Дамерау-Левенштейна и клавиатурных дистанций.
   - Встроенный словарь популярных разговорных опечаток ($O(1)$) для русского и английского языков.
   - Канонический регистр аббревиатур (`OST`, `macOS`, `СПб`, `СНиП`, `хз`, `спс`).
   - Принцип FlorisBoard: отмена автозамены одним нажатием Backspace с сохранением слова.
   - Защита словаря: опечатки не засоряют пользовательский лексикон.
3. **Мультиязычность из коробки (11 языков)**:
   - Русский (ЙЦУКЕН с 33 клавишами), Английский (QWERTY), Немецкий, Французский, Испанский, Португальский, Итальянский, Турецкий, Украинский, Белорусский, Казахский.
   - Автоматическое определение языка устройства при первом запуске (для RU $\rightarrow$ RU + EN, для остальных $\rightarrow$ язык системы + EN).
   - Быстрое переключение раскладок свайпом влево/вправо по пробелу или клавишей глобуса.
4. **Жесты и управление курсором**:
   - **Свайп по пробелу**: плавное и точное перемещение текстового курсора без случайных нажатий.
   - **Свайп влево от Backspace**: мгновенное удаление целого слова.
   - **Long-Press (удержание)**: всплывающее меню акцентов, цифр и спецсимволов.
   - Двойной тап по Shift для CapsLock с визуальным индикатором.
5. **Эмодзи-панель Android 15.1**:
   - 6 структурированных категорий с удобной сеткой и переключателем.
   - Высота панели оптимизирована под современные соотношения сторон экранов.
   - Текстовые шорткаты (`:)` $\rightarrow$ 😊, `<3` $\rightarrow$ ❤️).
6. **Темы оформления**:
   - **Rynk Dark (Cyan)** — фирменный графитовый стиль с неоновым акцентом.
   - **Rynk Light (Sapphire)** — чистая жемчужная тема с сапфировым акцентом.
   - **AMOLED Black** — абсолютный чёрный (`#000000`) для экономии батареи на OLED-экранах.
   - **Sunset Twilight** — тёплая вечерняя палитра.
7. **Конфиденциальность 100%**:
   - Никаких сетевых разрешений (`INTERNET` отсутствует в манифесте).
   - Ваши пароли, сообщения и персональные данные никогда не покидают устройство.

---

### 🏗️ Архитектура

```
Rynk/
├── rust-core/                        # ОСНОВНОЕ ВЫСОКОПРОИЗВОДИТЕЛЬНОЕ ЯДРО (RUST)
│   ├── Cargo.toml                    # cdylib + rlib конфигурация
│   └── src/
│       ├── lib.rs                    # Модуль интеграции и комплексные unit-тесты
│       ├── jni_bridge.rs             # Безнакладной JNI FFI интерфейс к Android SDK
│       ├── keyboard/
│       │   ├── mod.rs                # Координатор движка KeyboardEngine
│       │   ├── key.rs                # Описание геометрии, типов и кодов клавиш
│       │   ├── layout.rs             # Генератор раскладок (RU, EN, 123, символы)
│       │   ├── state.rs              # Состояние ввода (Shift, Caps, язык, события)
│       │   └── touch.rs              # Мультитач-трекер, жесты курсора и лонг-пресс
│       ├── render/
│       │   ├── mod.rs                # Главный конвейер отрисовки кадра
│       │   ├── canvas.rs             # Прямой 2D-растеризатор, SDF-сглаживание и тени
│       │   ├── font.rs               # Векторный рендерер букв и системных иконок
│       │   ├── popup.rs              # Всплывающие превью и меню альтернатив
│       │   ├── animation.rs          # Плавные тайминги нажатия и ripple-эффектов
│       │   └── theme.rs              # Цветовые палитры и стилизация тем
│       ├── prediction/
│       │   ├── mod.rs                # Сервис подсказок и предсказания
│       │   ├── trie.rs               # Высокоэффективное дерево префиксов
│       │   ├── dictionary.rs         # Частотные словари для 11 языков + User Dict
│       │   ├── autocorrect.rs        # Ранжирование кандидатов по Левенштейну
│       │   ├── typos.rs              # Таблица частых орфографических опечаток
│       │   └── suggestions.rs        # Унифицированное ранжирование кандидатов
│       └── emoji/
│           ├── mod.rs                # Менеджер каталога и рендера эмодзи
│           └── data.rs               # База данных эмодзи Android 15.1
└── app/                              # ТОНКИЙ СЛОЙ ANDROID SDK (KOTLIN)
    ├── build.gradle.kts              # Автоматизированная сборка Rust NDK + AGP
    └── src/main/
        ├── AndroidManifest.xml       # Регистрация InputMethodService и активити
        ├── kotlin/org/rynk/keyboard/
        │   ├── NativeBridge.kt       # JNI-мост загрузки librynk_core.so
        │   ├── RynkKeyboardView.kt   # Hardware-accelerated View с Bitmap буфером
        │   ├── RynkInputMethodService.kt # Системный IME сервис Android
        │   ├── HapticManager.kt      # Тактильная отдача (VibrationEffect)
        │   ├── SetupWizardActivity.kt# Мастер первоначальной настройки
        │   └── SettingsActivity.kt   # Полнофункциональный экран настроек
        └── res/                      # Локализация на 11 языков и векторные ресурсы
```

---

### 🛠️ Сборка из исходного кода

#### Требования:
- **JDK**: OpenJDK 21 или 27
- **Android SDK & NDK**: NDK версия `26.3.11579264`
- **Rust Toolchain**: `rustup target add aarch64-linux-android x86_64-linux-android`

#### Команды сборки:
```bash
# 1. Прогон всех unit-тестов Rust ядра (14 тестов)
cargo test --manifest-path rust-core/Cargo.toml

# 2. Компиляция Rust библиотек и сборка Android APK
export JAVA_HOME=/usr/lib/jvm/java-27-openjdk
./gradlew buildRustCore assembleDebug

# Готовый APK будет расположен по пути:
# app/build/outputs/apk/debug/app-debug.apk
```

---

<a name="-in-english"></a>
## 🇬🇧 In English

### 🌟 Overview

**Rynk Keyboard** is a state-of-the-art, premium Android input method engineered for extreme responsiveness by combining **Rust** for the entire core processing and rendering with **Kotlin** strictly for mandatory Android SDK input method service bindings.

Unlike standard mobile keyboards burdened by heavy Java/Kotlin UI hierarchies or web runtimes, Rynk executes all layout math, multi-touch gesture processing, animation timing, predictive search, and 2D pixel rendering **directly in native Rust**. The resulting frame is flushed directly to the hardware frame buffer via `AndroidBitmap_lockPixels` (`libjnigraphics.so`) with zero memory copies.

---

### 🚀 Highlights & Features

1. **Zero-Latency Touch-to-Pixel (< 1 ms)**:
   - High-performance Signed Distance Fields (**SDF**) 2D rasterizer with subpixel anti-aliasing.
   - Smooth elevation drop shadows under keys for tactile realism.
   - Key-press micro-animations, popups, and ripple effects rendering up to 120 FPS.
2. **Next-Gen Autocorrect & Suggestions**:
   - In-memory **Trie** structure over 50,000+ words yielding lookups under 0.5 ms.
   - Weighted Damerau-Levenshtein distance calculation calibrated with word frequencies.
   - Direct $O(1)$ phonetic & colloquial typo lookup table for English and Russian (`thnaks` $\rightarrow$ `thanks`, `definately` $\rightarrow$ `definitely`, etc.).
   - Canonical abbreviation casing preservation (`OST`, `macOS`, `AFK`).
   - FlorisBoard-style instant undo: tapping Backspace once after spacebar restores the exact user input and prevents re-correction.
   - Clean user dictionary guarantee: uncorrected typos never pollute the dictionary.
3. **11 Supported Languages Out of the Box**:
   - English (QWERTY), Russian (ЙЦУКЕН), German, French, Spanish, Portuguese, Italian, Turkish, Ukrainian, Belarusian, Kazakh.
   - Automatic system locale adaptation on initial setup.
   - Instant language switching via spacebar swipe or dedicated globe key.
4. **Precision Gestures & Cursor Control**:
   - **Spacebar Swipe Cursor**: Glide left and right across the spacebar for character-accurate cursor positioning.
   - **Backspace Swipe**: Swipe left from Backspace to delete whole words in one stroke.
   - **Long-Press Diacritics**: Hold any key to reveal alternative characters, symbols, and digits.
   - Double-tap Shift for persistent CapsLock with visual state indicator.
5. **Android 15.1 Emoji Catalog**:
   - 6 categorized tabs with an ergonomic grid layout.
   - Expanded panel height matching modern high-aspect-ratio displays.
   - Instant inline emoji shortcuts (`:)` $\rightarrow$ 😊, `<3` $\rightarrow$ ❤️).
6. **Curated Themes**:
   - **Rynk Dark (Cyan)** — Signature graphite background with neon cyan accents.
   - **Rynk Light (Sapphire)** — Crisp pearl palette with deep sapphire highlights.
   - **AMOLED Black** — Pure `#000000` dark theme for maximum OLED battery savings.
   - **Sunset Twilight** — Warm gradient evening palette.
7. **100% Privacy by Design**:
   - Zero internet permissions in `AndroidManifest.xml`.
   - Your keystrokes, personal dictionary, and clipboard stay completely on-device.

---

### 🛠️ Building from Source

#### Prerequisites:
- **JDK**: OpenJDK 21 or 27
- **Android SDK & NDK**: NDK version `26.3.11579264`
- **Rust Toolchain**: `rustup target add aarch64-linux-android x86_64-linux-android`

#### Build Instructions:
```bash
# 1. Run full Rust core test suite (14 unit tests)
cargo test --manifest-path rust-core/Cargo.toml

# 2. Build native .so libraries and generate debug APK
export JAVA_HOME=/usr/lib/jvm/java-27-openjdk
./gradlew buildRustCore assembleDebug

# Output APK:
# app/build/outputs/apk/debug/app-debug.apk
```

---

### 📄 Лицензия / License

- **Русский:** Проект распространяется под свободной лицензией **Mozilla Public License Version 2.0 (MPL-2.0)**. Подробности в файле `LICENSE`.
- **English:** Distributed under the **Mozilla Public License Version 2.0 (MPL-2.0)**. See `LICENSE` for details.

Designed and crafted with passion for performance.
