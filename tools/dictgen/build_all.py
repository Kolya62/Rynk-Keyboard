#!/usr/bin/env python3
"""Builds app/src/main/assets/lm/<code>.rlm for every language in sources.json.

Run fetch_corpora.py first. Pass language codes to rebuild only those.
"""
import concurrent.futures as cf
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent
REPO = ROOT.parent.parent
CACHE = ROOT / ".cache"
CURATED = ROOT / "curated"
OUT = REPO / "app" / "src" / "main" / "assets" / "lm"
BIN = REPO / "target" / "release" / "dictgen"
# Curated file names that predate the language codes
CURATED_NAMES = {"zh-hans": "zh_cn", "zh-hant": "zh_tw", "yue": "zh_hk"}


def build(code: str, src: dict) -> str:
    cmd = [str(BIN), "--lang", code, "--tier", src["tier"], "--out", str(OUT / f"{code}.rlm")]
    for key in ("leipzig", "web"):
        if src.get(key):
            cmd += ["--sentences", str(CACHE / "leipzig" / f"{src[key]}-sentences.txt")]
    if src.get("tatoeba"):
        cmd += ["--dialog", str(CACHE / "tatoeba" / f"{src['tatoeba']}-sentences.txt")]
    if src.get("subtitles"):
        cmd += ["--subtitles", str(CACHE / "subtitles" / f"{src['subtitles']}.txt")]
    curated = CURATED / f"{CURATED_NAMES.get(code, code)}_words.txt"
    if curated.exists():
        cmd += ["--curated", str(curated)]
    bigrams = CURATED / f"{code}_bigrams.txt"
    if bigrams.exists():
        cmd += ["--bigrams", str(bigrams)]
    cmd += ["--multi-bigrams", str(CURATED / "multi_bigrams.txt")]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"{code}: {result.stderr.strip()}")
    return result.stderr.strip()


def main() -> int:
    subprocess.run(["cargo", "build", "--release", "-q", "-p", "dictgen"], cwd=REPO, check=True)
    OUT.mkdir(parents=True, exist_ok=True)
    sources = json.loads((ROOT / "sources.json").read_text())
    only = set(sys.argv[1:])
    todo = {c: s for c, s in sources.items() if not only or c in only}
    failed = 0
    # Tier A counts trigrams over millions of sentences and needs a few GB of RAM each
    heavy = {c: s for c, s in todo.items() if s["tier"] == "A"}
    light = {c: s for c, s in todo.items() if s["tier"] != "A"}
    for group, workers in ((heavy, 2), (light, max(1, (os.cpu_count() or 2) // 2))):
        with cf.ThreadPoolExecutor(workers) as ex:
            futures = {ex.submit(build, c, s): c for c, s in group.items()}
            for f in cf.as_completed(futures):
                try:
                    print(f.result(), flush=True)
                except Exception as e:  # noqa: BLE001 - keep building other languages
                    failed += 1
                    print(f"FAILED {e}", flush=True)
    total = sum(p.stat().st_size for p in OUT.glob("*.rlm"))
    print(f"total: {total / 1e6:.1f} MB in {OUT}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
