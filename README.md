<div align="center">

# Rynk Keyboard

**Android-клавиатура с ядром на Rust: автозамена с учётом касаний, контекстные подсказки, свайп-набор, 85 языков, без доступа к интернету**

*Android keyboard with a Rust core: touch-aware autocorrect, context predictions, glide typing, 85 languages, no internet access*

[![Platform](https://img.shields.io/badge/Android-7.0%2B%20(API%2024--35)-brightgreen?style=for-the-badge&logo=android)](https://developer.android.com)
[![Core](https://img.shields.io/badge/Core-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Release](https://img.shields.io/badge/APK-v1.1.0-success?style=for-the-badge&logo=android)](https://github.com/Kolya62/Rynk-Keyboard/releases/latest)
[![License](https://img.shields.io/badge/License-MPL%202.0-blue?style=for-the-badge)](LICENSE)

[Русский](#русский) • [English](#english)

</div>

---

## Русский

### О проекте

Rynk — системная клавиатура для Android. Вся логика работает в нативном ядре на **Rust**: раскладки, обработка касаний, отрисовка кадра в Bitmap, словари, автозамена, подсказки и свайп-набор. Слой на **Kotlin** отвечает только за связь с Android: `InputMethodService`, вывод кадра, настройки, буфер обмена и вибрацию.

### Возможности

**Набор и исправление**
- **Автозамена с учётом касаний.** Каждое нажатие оценивается по расстоянию до центров клавиш (2D-гауссиана), поэтому касание на границе двух клавиш легко переосмыслить. Учитываются пропущенные, лишние и переставленные буквы, диакритика (`dziekuje` → `dziękuję`), е/ё и типичные фонетические ошибки.
- **Контекст.** Вероятность слова считается по двум предыдущим словам по корпусным триграммам и биграммам с откатом. Пример: «я живу в мосвке» → «Москве».
- **Пропущенный пробел:** «приветкак» → «привет как».
- **Сила автозамены:** выключена / мягкая / обычная / активная. Backspace сразу после исправления возвращает набранное, а отменённое слово запоминается.
- **Центральная подсказка** всегда совпадает с тем, что вставит пробел.
- **Подсказки следующего слова**, в том числе в начале предложения, с правильным регистром.
- **Текст из поля ввода.** При запуске и после перемещения курсора клавиатура читает текст перед курсором: можно продолжить начатое слово, а заглавная буква ставится по правилам поля.
- **Обучение на вашем наборе.** Слова и пары слов запоминаются со счётчиками и затуханием и с учётом языка. Есть пользовательский словарь с экспортом и импортом в текстовый файл.
- **Умная пунктуация и авто-заглавные.** Обе функции отключаются в полях email, URL, паролей и чисел.
- **Enter** выполняет действие поля (поиск, отправить, далее, готово) и показывает соответствующую иконку.

**Жесты и инструменты**
- **Свайп-набор**: ведите пальцем по буквам. Слово вставляется целиком, альтернативы показываются в подсказках. Пробел между словами ставится автоматически, Backspace удаляет слово целиком. На экране виден след пальца.
- **Курсор:** удерживайте пробел и ведите пальцем.
- **Удаление слов:** ведите влево от Backspace — слова выделяются с предпросмотром. При отпускании они удаляются, возврат пальца назад отменяет.
- **Свайп по пробелу** меняет язык. Действие двойного пробела настраивается: сменить язык, поставить точку или ничего.
- **Панель правки:** стрелки, режим выделения, «выделить всё», копировать, вырезать, вставить, отменить, повторить.
- **История буфера обмена**: до 50 записей, закрепление, автоочистка незакреплённых (1 час, 1 день или 7 дней). Есть отдельная подсказка «вставить» для только что скопированного текста.
- **Ряд цифр, режим одной руки, высота клавиатуры 80–130 %.**
- **Голосовой ввод**: кнопка передаёт управление системному голосовому вводу.
- **Эмодзи** Unicode 16.0 по категориям, с поиском на русском и английском. Текстовые шорткаты: `:)` → 😊, `<3` → ❤️.
- **Темы:** светлая, графит, AMOLED чёрная, закат. Вибрация с регулировкой силы, звук нажатий.

**Языки**
- **85 языков.** Для каждого есть языковая модель: словарь до 120–135 тыс. слов у основных языков и около 40 тыс. у большинства остальных (меньше у языков с небольшими корпусами), а также биграммы. У 10 основных языков (ru, en, uk, de, fr, es, pt, it, pl, tr) есть и триграммы.
- Корейский — раскладка 2-Set с автоматической сборкой слогов. Китайский и японский вводятся через латиницу (пиньинь и ромадзи).

**Приватность**
- У приложения нет разрешения `INTERNET`.
- В полях паролей и в режиме инкогнито (`IME_FLAG_NO_PERSONALIZED_LEARNING`) текст не читается, подсказки, обучение и буфер обмена отключены.
- Выученные слова, пользовательский словарь и история буфера исключены из облачных резервных копий и переноса на другое устройство.
- Копии, помеченные системой как чувствительные (Android 13+, менеджеры паролей), в историю не попадают.

### Качество (замеры)

Замеры сделаны на компьютере по оценочным стендам из `tools/dictgen/examples`. Стенд «печатает» фразы из Tatoeba симулированными касаниями с разбросом пальца и случайными пропусками или перестановками букв.

| | Русский | Английский |
|---|---|---|
| Исправлено опечаток при небрежном наборе | 81 % | 71 % |
| Испорчено верно набранных слов | 0,1 % | 0 % |
| Свайп-набор, первое слово верное | 92–96 % | 94–97 % |
| Свайп-набор, верное слово в первой тройке | 98–99 % | 99–99,9 % |
| Подсказки после нажатия, p95 | ≈ 0,6 мс | ≈ 0,45 мс |

На телефоне время выше, чем на компьютере.

### Известные ограничения

- **Нет родной раскладки** у 25 языков с нелатинской письменностью: bg, mk, ky, tg, mn, el, hy, ka, hi, mr, ne, bn, pa, gu, ta, te, kn, ml, si, th, my, km, am. Для них показывается латинская QWERTY. У казахской, персидской и урду раскладок не хватает части букв. Языковые модели для этих языков уже собраны и заработают, как только появятся раскладки.
- **Переводы.** Новые строки настроек переведены только на русский и английский, остальные локали показывают английский.
- **Размер.** APK весит около 50 МБ, из них около 44 МБ — языковые модели.

### Архитектура

```
Rynk/
├── rust-core/src/                 Ядро (Rust)
│   ├── jni_bridge.rs              JNI-интерфейс, бинарный протокол событий
│   ├── keyboard/
│   │   ├── mod.rs                 KeyboardEngine: состояние, действия клавиш, жесты
│   │   ├── layout.rs              Раскладки, ряд цифр, режим одной руки, панель правки
│   │   ├── touch.rs               Мультитач: долгое нажатие, курсор, выделение, свайп-набор
│   │   ├── context.rs             Разбор текста перед курсором
│   │   ├── toolbar.rs             Строка инструментов
│   │   ├── clipboard_panel.rs     Панель истории буфера обмена
│   │   ├── hangul.rs              Сборка корейских слогов
│   │   ├── key.rs, state.rs       Клавиши, состояние, события, настройки движка
│   ├── prediction/
│   │   ├── lm_data.rs             Формат языковых моделей (.rlm)
│   │   ├── model_source.rs        Загрузка моделей из assets (AAssetManager)
│   │   ├── lexicon.rs             Компактный словарь с префиксным поиском
│   │   ├── dictionary.rs          Языки, пользовательские слова
│   │   ├── lm.rs                  Модель контекста (триграммы, откат)
│   │   ├── decoder.rs             Декодер опечаток с учётом касаний
│   │   ├── correction.rs          Ранжирование и решение об автозамене
│   │   ├── gesture.rs             Декодер свайп-набора
│   │   ├── adaptive.rs            Обучение на наборе пользователя
│   │   ├── suggestions.rs         Строка подсказок
│   │   └── typos.rs, morphology.rs, cjk.rs, autocorrect.rs
│   ├── render/                    Растеризация клавиш, тени, анимации, темы
│   └── emoji/                     Каталог эмодзи и поиск
├── app/src/main/
│   ├── kotlin/org/rynk/keyboard/  IME-сервис, View, настройки, история буфера
│   ├── assets/lm/*.rlm            Языковые модели 85 языков
│   └── res/                       Ресурсы, экраны настроек, локализация
└── tools/dictgen/                 Генератор языковых моделей и стенды оценки
```

### Сборка

**Требования:** JDK 21+, Android SDK, NDK `26.3.11579264`, Rust с целевыми платформами:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

```bash
# Тесты ядра
cargo test --release -p rynk_core

# Тесты Android, сборка APK (Gradle сам собирает Rust под все ABI)
./gradlew testDebugUnitTest assembleRelease
# → app/build/outputs/apk/release/app-release.apk
```

Подпись релиза берётся из переменных окружения `RYNK_KEYSTORE_PATH`, `RYNK_KEYSTORE_PASSWORD`, `RYNK_KEY_ALIAS`, `RYNK_KEY_PASSWORD` или из `local.properties` (пример в `local.properties.example`).

### Языковые модели

Готовые модели лежат в `app/src/main/assets/lm`. Пересобрать их:

```bash
python3 tools/dictgen/fetch_corpora.py   # около 6 ГБ корпусов в tools/dictgen/.cache
python3 tools/dictgen/build_all.py       # все языки или, например: build_all.py ru en

# Оценка качества
cargo run --release -p dictgen --example eval_autocorrect -- ru tools/dictgen/.cache/tatoeba/rus-sentences.txt
cargo run --release -p dictgen --example eval_gesture -- ru tools/dictgen/.cache/tatoeba/rus-sentences.txt
```

Подробности — в [tools/dictgen/README.md](tools/dictgen/README.md).

### Данные и лицензии

Код распространяется по лицензии **MPL-2.0** (см. [LICENSE](LICENSE)). Языковые модели построены по данным Leipzig Corpora Collection (CC BY), Tatoeba (CC BY 2.0 FR) и FrequencyWords / OpenSubtitles (CC BY-SA 4.0). Файлы моделей распространяются по лицензии **CC BY-SA 4.0**, подробности в [DATA_LICENSES.md](DATA_LICENSES.md).

---

## English

### About

Rynk is a system keyboard for Android. All the logic lives in a native **Rust** core: layouts, touch handling, drawing the frame into a bitmap, dictionaries, autocorrect, suggestions and glide typing. The **Kotlin** layer only connects it to Android: `InputMethodService`, frame output, settings, clipboard and vibration.

### Features

**Typing and correction**
- **Touch-aware autocorrect.** Every tap is scored against the key centers with a 2D Gaussian, so a tap on the border of two keys is cheap to reinterpret. It also handles missing, extra and swapped letters, diacritics (`dziekuje` → `dziękuję`) and common phonetic mistakes.
- **Context.** Word probability comes from the two previous words, using corpus trigrams and bigrams with backoff. Example: "я живу в мосвке" → "Москве".
- **Missing space fix:** "helloworld" → "hello world".
- **Autocorrect strength:** off / mild / normal / aggressive. Backspace right after a correction restores what you typed, and the rejected word is remembered.
- **The center suggestion** is always what space will commit.
- **Next-word predictions**, including at the start of a sentence, with correct capitalization.
- **Editor text.** On input start and after cursor moves the keyboard reads the text before the cursor, so you can continue a word you started, and capitals follow the field's rules.
- **Learning from your typing.** Words and word pairs are remembered with counts, decay and their language. There is a personal dictionary with text-file export and import.
- **Smart punctuation and auto-capitalization.** Both are off in email, URL, password and number fields.
- **Enter** performs the field's action (search, send, next, done) and shows its icon.

**Gestures and tools**
- **Glide typing**: slide across the letters. The word is entered whole, with alternatives in the suggestion bar. The next word gets its space automatically, and backspace deletes the gestured word. A finger trail is shown on screen.
- **Cursor:** hold the space bar and slide.
- **Deleting words:** drag left from backspace — words are selected with a preview. Releasing deletes them, dragging back cancels.
- **Space swipe** switches the language. The double-space action is configurable: switch language, insert a period, or nothing.
- **Editing panel:** arrows, selection mode, select all, copy, cut, paste, undo, redo.
- **Clipboard history**: up to 50 entries, pinning, automatic expiry of unpinned ones (1 hour, 1 day or 7 days). There is also a separate paste suggestion for freshly copied text.
- **Number row, one-handed mode, keyboard height 80–130%.**
- **Voice input**: the button hands over to the system voice input.
- **Emoji** (Unicode 16.0) by category, with search in Russian and English. Text shortcuts: `:)` → 😊, `<3` → ❤️.
- **Themes:** light, graphite, AMOLED black, sunset. Vibration with adjustable strength, key click sound.

**Languages**
- **85 languages.** Each has a language model: up to 120–135k words for the main languages and about 40k for most others (fewer where corpora are small), plus bigrams. The 10 main languages (ru, en, uk, de, fr, es, pt, it, pl, tr) also have trigrams.
- Korean uses a 2-Set layout with automatic syllable composition. Chinese and Japanese are typed in Latin letters (pinyin and romaji).

**Privacy**
- The app has no `INTERNET` permission.
- In password and incognito fields (`IME_FLAG_NO_PERSONALIZED_LEARNING`) no text is read, and suggestions, learning and the clipboard are off.
- Learned words, the personal dictionary and clipboard history are excluded from cloud backup and device transfer.
- Clips the system marks as sensitive (Android 13+, password managers) are never stored in the history.

### Quality (measured)

Measured on a desktop with the evaluation tools in `tools/dictgen/examples`. They type Tatoeba sentences with simulated touches, spreading the finger around each key and occasionally dropping or swapping letters.

| | Russian | English |
|---|---|---|
| Typos fixed, sloppy typing | 81% | 71% |
| Correctly typed words broken | 0.1% | 0% |
| Glide typing, first word correct | 92–96% | 94–97% |
| Glide typing, correct word in top 3 | 98–99% | 99–99.9% |
| Suggestions after a keystroke, p95 | ≈ 0.6 ms | ≈ 0.45 ms |

Times are higher on a phone.

### Known limitations

- **No native layout** for 25 non-Latin-script languages: bg, mk, ky, tg, mn, el, hy, ka, hi, mr, ne, bn, pa, gu, ta, te, kn, ml, si, th, my, km, am. They show a Latin QWERTY. The Kazakh, Persian and Urdu layouts are missing some letters. The language models for these languages are already built and will work once the layouts exist.
- **Translations.** New settings strings are translated into Russian and English only; other locales show English.
- **Size.** The APK is about 50 MB, about 44 MB of which are language models.

### Architecture

See the tree in the Russian section: `rust-core/src/keyboard` (engine, layouts, touch and gestures, panels), `rust-core/src/prediction` (models, lexicon, context model, touch-aware decoder, autocorrect decision, glide decoder, learning), `rust-core/src/render`, `rust-core/src/emoji`, `app/` (Kotlin IME layer, settings, clipboard history, `assets/lm` models) and `tools/dictgen/` (model builder and evaluation tools).

### Building

**Requirements:** JDK 21+, Android SDK, NDK `26.3.11579264`, and Rust with the Android targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

```bash
cargo test --release -p rynk_core                 # core tests
./gradlew testDebugUnitTest assembleRelease       # Gradle builds Rust for every ABI
# → app/build/outputs/apk/release/app-release.apk
```

Release signing comes from the `RYNK_KEYSTORE_PATH`, `RYNK_KEYSTORE_PASSWORD`, `RYNK_KEY_ALIAS` and `RYNK_KEY_PASSWORD` environment variables, or from `local.properties` (see `local.properties.example`).

Language models are rebuilt with `tools/dictgen` (`fetch_corpora.py`, then `build_all.py`); see [tools/dictgen/README.md](tools/dictgen/README.md).

### Data and licenses

The code is licensed under **MPL-2.0** (see [LICENSE](LICENSE)). The language models are built from the Leipzig Corpora Collection (CC BY), Tatoeba (CC BY 2.0 FR) and FrequencyWords / OpenSubtitles (CC BY-SA 4.0). The model files are distributed under **CC BY-SA 4.0**; see [DATA_LICENSES.md](DATA_LICENSES.md).

---

### Поддержать автора / Support the author

Банковская карта / Bank card: `4466 1481 2794 9960`
