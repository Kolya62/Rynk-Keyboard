# dictgen

Builds the per-language model files `app/src/main/assets/lm/<code>.rlm` that the keyboard loads
lazily: word list with frequencies and canonical spelling, bigrams, and (tier A) trigrams.
The format is defined in `rust-core/src/prediction/lm_data.rs`.

```bash
python3 tools/dictgen/fetch_corpora.py      # ~6.2 GB into tools/dictgen/.cache (git-ignored)
python3 tools/dictgen/build_all.py          # all languages, or e.g.: build_all.py ru en
cargo run --release -p dictgen --example inspect -- app/src/main/assets/lm/ru.rlm как спасибо
cargo run --release -p dictgen --example load_bench
cargo run --release -p dictgen --example context_demo -- ru "я иду в " "спасибо за "
```

- Data per language (`sources.json`): a Leipzig news/wiki corpus, a Leipzig web corpus for the
  main languages, Tatoeba dialog sentences (counted double: closest to what people type; its
  placeholder names Tom/Mary are dropped) and OpenSubtitles word frequencies.
- Trigrams are kept only when the second context word changes the prediction (≥ 1.5× the
  bigram probability), so the budget goes to informative context.
- `sources.json` — corpora per language and the tier (A: 120k words + trigrams, B: 40k words,
  C: curated list only, for languages without word spacing or without a usable corpus).
- `curated/` — hand-maintained lists: slang and new words (frequency capped below the core
  vocabulary when the corpus lacks them), canonical spellings (`macOS`, `СПб`) and hand-picked
  bigrams. Edit these, then rebuild.
- Only words typeable on the language's layout are kept. Where the layout cannot type the
  language's script yet (or covers < 98% of its letters), words are kept by script so the model
  is ready when a native layout lands. Known misspellings from `typos.rs` are excluded.

Data licenses: see `DATA_LICENSES.md` at the repository root.
