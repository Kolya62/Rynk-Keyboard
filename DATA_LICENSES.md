# Data licenses

The application code is licensed under MPL-2.0 (see `LICENSE`). The language model files in
`app/src/main/assets/lm/*.rlm` are derived works of the corpora below and are distributed under
**CC BY-SA 4.0** (required by the share-alike terms of the OpenSubtitles frequency lists).

They are built by `tools/dictgen` (`fetch_corpora.py`, then `build_all.py`); the exact corpus
used for each language is listed in `tools/dictgen/sources.json`.

## Sources

- **Leipzig Corpora Collection**, Universität Leipzig — news, Wikipedia, web and community
  corpora, licensed under [CC BY](https://creativecommons.org/licenses/by/4.0/).
  D. Goldhahn, T. Eckart, U. Quasthoff: *Building Large Monolingual Dictionaries at the Leipzig
  Corpora Collection: From 100 to 200 Languages.* LREC 2012.
  <https://wortschatz.uni-leipzig.de/en/download>

- **FrequencyWords** by Hermit Dave, word frequency lists built from OpenSubtitles 2018
  (P. Lison, J. Tiedemann: *OpenSubtitles2016: Extracting Large Parallel Corpora from Movie and
  TV Subtitles.* LREC 2016), licensed under
  [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
  <https://github.com/hermitdave/FrequencyWords>

- **Rynk curated lists** (`tools/dictgen/curated/`) — slang, abbreviations and canonical
  spellings maintained in this repository, MPL-2.0.
