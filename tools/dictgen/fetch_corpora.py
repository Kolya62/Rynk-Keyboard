#!/usr/bin/env python3
"""Downloads the corpora listed in sources.json into tools/dictgen/.cache.

Leipzig Corpora Collection (CC BY): news/wiki corpus per language plus a web corpus for the
main languages ("web" in sources.json); only the *-sentences.txt file is kept from each archive.
Tatoeba (CC BY 2.0 FR): short everyday sentences, used as conversational n-gram data.
FrequencyWords / OpenSubtitles 2018 (CC BY-SA 4.0): *_50k.txt (or *_full.txt) frequency lists.
"""
import concurrent.futures as cf
import json
import pathlib
import subprocess
import sys
import bz2
import tarfile

ROOT = pathlib.Path(__file__).resolve().parent
CACHE = ROOT / ".cache"
LEIPZIG_URL = "https://downloads.wortschatz-leipzig.de/corpora/{}.tar.gz"
TATOEBA_URL = "https://downloads.tatoeba.org/exports/per_language/{0}/{0}_sentences.tsv.bz2"
SUBTITLES_URL = "https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/{0}/{0}_{1}.txt"


def download(url: str, dest: pathlib.Path) -> None:
    tmp = dest.with_suffix(dest.suffix + ".part")
    subprocess.run(["curl", "-sSfL", "--retry", "3", "-o", str(tmp), url], check=True)
    tmp.rename(dest)


def fetch_leipzig(name: str) -> str:
    sentences = CACHE / "leipzig" / f"{name}-sentences.txt"
    if sentences.exists():
        return f"{name}: cached"
    archive = CACHE / "leipzig" / f"{name}.tar.gz"
    if not archive.exists():
        download(LEIPZIG_URL.format(name), archive)
    with tarfile.open(archive) as tar:
        member = next(m for m in tar.getmembers() if m.name.endswith("-sentences.txt"))
        with tar.extractfile(member) as src, open(sentences, "wb") as dst:
            dst.write(src.read())
    archive.unlink()
    return f"{name}: ok"


def fetch_tatoeba(iso: str) -> str:
    sentences = CACHE / "tatoeba" / f"{iso}-sentences.txt"
    if sentences.exists():
        return f"{iso} tatoeba: cached"
    archive = CACHE / "tatoeba" / f"{iso}.tsv.bz2"
    download(TATOEBA_URL.format(iso), archive)
    with bz2.open(archive, "rb") as src, open(sentences, "wb") as dst:
        dst.write(src.read())
    archive.unlink()
    return f"{iso} tatoeba: ok"


def fetch_subtitles(code: str) -> str:
    dest = CACHE / "subtitles" / f"{code}.txt"
    if dest.exists():
        return f"{code} subtitles: cached"
    legacy = CACHE / "subtitles" / f"{code}_50k.txt"
    if legacy.exists():
        legacy.rename(dest)
        return f"{code} subtitles: cached"
    # Small languages only have the full list
    for variant in ("50k", "full"):
        try:
            download(SUBTITLES_URL.format(code, variant), dest)
            return f"{code} subtitles ({variant}): ok"
        except subprocess.CalledProcessError:
            continue
    raise RuntimeError(f"{code}: no subtitle frequency list")


def main() -> int:
    sources = json.loads((ROOT / "sources.json").read_text())
    only = set(sys.argv[1:])
    (CACHE / "leipzig").mkdir(parents=True, exist_ok=True)
    (CACHE / "subtitles").mkdir(parents=True, exist_ok=True)
    (CACHE / "tatoeba").mkdir(parents=True, exist_ok=True)
    jobs = []
    with cf.ThreadPoolExecutor(3) as ex:
        for code, src in sources.items():
            if only and code not in only:
                continue
            for key in ("leipzig", "web"):
                if src.get(key):
                    jobs.append(ex.submit(fetch_leipzig, src[key]))
            if src.get("subtitles"):
                jobs.append(ex.submit(fetch_subtitles, src["subtitles"]))
            if src.get("tatoeba"):
                jobs.append(ex.submit(fetch_tatoeba, src["tatoeba"]))
        failed = 0
        for job in cf.as_completed(jobs):
            try:
                print(job.result(), flush=True)
            except Exception as e:  # noqa: BLE001 - report and continue with other languages
                failed += 1
                print(f"FAILED: {e}", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
