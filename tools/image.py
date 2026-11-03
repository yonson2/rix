#!/usr/bin/env python3
"""Generate an image through EvoLink ([050]): the app icon candidates.

Content time only: the game never calls it. Each call writes NAME.png and a sidecar
NAME.json with the request and the finished task, so a pick traces back to exactly what
produced it. Seedream has no seed: the tracked PNG is the source, the sidecar its record.

Key from the environment: EVOLINK_TOKEN (Rix keeps it in .env, D16).

Usage: tools/image.py PROMPT --out DIR/NAME [--model M] [--size 1:1] [--quality 2K]
"""

import argparse
import json
import os
import time
import urllib.error
import urllib.request
from pathlib import Path

EVOLINK = "https://api.evolink.ai/v1"


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


def image_urls(node):
    if isinstance(node, dict):
        node = list(node.values())
    if isinstance(node, list):
        for v in node:
            yield from image_urls(v)
    elif isinstance(node, str) and node.startswith("http"):
        yield node


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("prompt")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--model", default="doubao-seedream-5.0-pro")
    ap.add_argument("--size", default="1:1")
    ap.add_argument("--quality", default="2K")
    args = ap.parse_args()
    body = {"model": args.model, "prompt": args.prompt, "size": args.size,
            "quality": args.quality, "output_format": "png"}
    started = time.time()
    status, raw = call(f"{EVOLINK}/images/generations", body)
    if status != 200:
        raise SystemExit(f"image failed: HTTP {status} {raw[:500]!r}")
    task = json.loads(raw)["id"]
    print(f"  task {task}")
    deadline = time.time() + 600
    while True:
        time.sleep(5)
        status, raw = call(f"{EVOLINK}/tasks/{task}")
        detail = json.loads(raw) if status == 200 else {}
        if detail.get("status") in ("completed", "failed", "cancelled"):
            break
        if time.time() > deadline:
            raise SystemExit(f"image: task {task} still {detail.get('status')}")
    if detail["status"] != "completed":
        raise SystemExit(f"image: task {task} {detail['status']}: {json.dumps(detail)[:500]}")
    # Links expire after 24 hours: download at once.
    url = next(image_urls(detail.get("results", detail)))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=300) as res:
        args.out.with_suffix(".png").write_bytes(res.read())
    meta = {"provider": "evolink", "request": body, "task": detail,
            "elapsed_s": round(time.time() - started, 1),
            "created": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    args.out.with_suffix(".json").write_text(json.dumps(meta, indent=2) + "\n")
    print(f"{args.out.with_suffix('.png')} in {meta['elapsed_s']} s")


if __name__ == "__main__":
    main()
