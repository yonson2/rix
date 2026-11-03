#!/usr/bin/env python3
"""Generate music with Suno v6 through EvoLink ([043], D22); the Suno part of Ketania's
tools/gen-music.py.

Content time only: the game never calls Suno. One call returns two takes (a and b). Each call
writes the takes' masters (EvoLink serves MP3) and a sidecar NAME.json with the request and
the finished task, so a pick traces back to exactly what produced it.

Key from the environment: EVOLINK_TOKEN (Rix keeps it in .env, D16).

Usage: tools/suno.py BRIEFS.json BRIEF_ID --out DIR [--calls N]

A brief: {"id", "title", "styles": [...], "negative": [...], "bpm", "key",
"sections": [{"name", "seconds"}]}.
"""

import argparse
import json
import os
import time
import urllib.error
import urllib.request
from pathlib import Path

EVOLINK = "https://api.evolink.ai/v1"
MODEL = "suno-v6-beta"


def call(url, body=None, timeout=120):
    data = json.dumps(body).encode() if body is not None else None
    headers = {"Authorization": f"Bearer {os.environ['EVOLINK_TOKEN']}",
               "Content-Type": "application/json"}
    req = urllib.request.Request(url, data=data, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as res:
            return res.status, res.read()
    except urllib.error.HTTPError as err:
        return err.code, err.read()


def request(brief):
    return {
        "model": MODEL, "custom_mode": True, "instrumental": True,
        "title": brief["title"],
        "style": ", ".join(brief["styles"] + [f"{brief['bpm']}bpm", brief["key"], "instrumental"]),
        "negative_tags": ", ".join(brief["negative"])[:200],
        "prompt": "\n".join(f"[{s['name']}]" for s in brief["sections"]),
        "duration": sum(s["seconds"] for s in brief["sections"]),
    }


def audio_urls(node):
    if isinstance(node, dict):
        for k, v in node.items():
            if k == "audio_url" and isinstance(v, str) and v.startswith("http"):
                yield v
            elif isinstance(v, (dict, list)):
                yield from audio_urls(v)
    elif isinstance(node, list):
        for v in node:
            yield from audio_urls(v)


def generate(brief, stem: Path):
    body = request(brief)
    started = time.time()
    status, raw = call(f"{EVOLINK}/audios/generations", body)
    if status != 200:
        raise SystemExit(f"suno failed: HTTP {status} {raw[:500]!r}")
    task = json.loads(raw)["id"]
    print(f"  task {task}")
    deadline = time.time() + 1200
    while True:
        time.sleep(15)
        status, raw = call(f"{EVOLINK}/tasks/{task}")
        detail = json.loads(raw) if status == 200 else {}
        if detail.get("status") in ("completed", "failed", "cancelled"):
            break
        if time.time() > deadline:
            raise SystemExit(f"suno: task {task} still {detail.get('status')}")
    if detail["status"] != "completed":
        raise SystemExit(f"suno: task {task} {detail['status']}: {json.dumps(detail)[:500]}")
    # Links last 72 hours: download at once.
    files = []
    for take, url in zip("ab", dict.fromkeys(audio_urls(detail))):
        out = stem.with_name(f"{stem.name}{take}.mp3")
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=300) as res:
            out.write_bytes(res.read())
        files.append(out.name)
    meta = {"provider": "suno via evolink", "brief": brief, "request": body, "task": detail,
            "files": files, "elapsed_s": round(time.time() - started, 1),
            "created": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    stem.with_name(stem.name + ".json").write_text(json.dumps(meta, indent=2) + "\n")
    print(f"{stem}: {', '.join(files)} in {meta['elapsed_s']} s")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("briefs", type=Path)
    ap.add_argument("brief_id")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--calls", type=int, default=1)
    args = ap.parse_args()
    brief = next(b for b in json.loads(args.briefs.read_text()) if b["id"] == args.brief_id)
    args.out.mkdir(parents=True, exist_ok=True)
    for n in range(1, args.calls + 1):
        generate(brief, args.out / f"{brief['id']}-{n}")


if __name__ == "__main__":
    main()
