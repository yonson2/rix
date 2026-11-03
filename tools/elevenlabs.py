#!/usr/bin/env python3
"""Generate sound effects with ElevenLabs ([042]); the sfx part of Ketania's tools/elevenlabs.py.

Content time only: the game never calls ElevenLabs. Every call writes its audio master and a
sidecar JSON beside it (NAME.json): the exact request, the model, the output format the plan
accepted, the billing and request headers, and the timing, so a pick traces back to exactly
what produced it. Masters are lossless where the plan allows (44.1 kHz PCM, else MP3).

Key from the environment: ELEVENLABS_TOKEN (Rix keeps it in .env, D16).

Usage:
  tools/elevenlabs.py sfx PROMPT --out DIR/NAME [--seconds S] [--influence P] [--loop] [--takes N]
"""

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
import wave
from pathlib import Path

API = "https://api.elevenlabs.io"
SFX_MODEL = "eleven_text_to_sound_v2"
# Best first; a tier refusal moves down the ladder.
PCM_LADDER = ["pcm_44100", "mp3_44100_192", "mp3_44100_128"]
KEPT_HEADERS = ("character-cost", "request-id", "history-item-id", "x-trace-id", "content-type")


def call(path, body=None, method="POST"):
    """One API call, retried with backoff on 429 and 5xx."""
    data = json.dumps(body).encode() if body is not None else None
    headers = {"xi-api-key": os.environ["ELEVENLABS_TOKEN"], "Content-Type": "application/json"}
    for attempt in range(6):
        req = urllib.request.Request(f"{API}{path}", data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=300) as res:
                return res.status, {k.lower(): v for k, v in res.headers.items()}, res.read()
        except urllib.error.HTTPError as err:
            status, raw = err.code, err.read()
            if status != 429 and status < 500:
                return status, {k.lower(): v for k, v in err.headers.items()}, raw
            wait = float(err.headers.get("retry-after") or 2 ** attempt)
            print(f"  HTTP {status}, retrying in {wait:.0f} s", file=sys.stderr)
            time.sleep(wait)
    raise SystemExit(f"{path}: still failing after retries")


def call_ladder(path, body, ladder):
    """Calls with the best output format the plan accepts."""
    for fmt in ladder:
        status, headers, raw = call(f"{path}?output_format={fmt}", body)
        if status == 200:
            return fmt, headers, raw
        text = raw.decode(errors="replace")
        print(f"  {fmt}: HTTP {status} {text[:200]}", file=sys.stderr)
        if status not in (400, 401, 403, 422):
            break
    raise SystemExit(f"{path}: no output format accepted")


def write_master(stem: Path, fmt: str, raw: bytes, channels: int) -> Path:
    if fmt.startswith("pcm_"):
        out = stem.with_name(stem.name + ".wav")
        with wave.open(str(out), "wb") as w:
            w.setnchannels(channels)
            w.setsampwidth(2)
            w.setframerate(int(fmt.split("_")[1]))
            w.writeframes(raw)
        return out
    out = stem.with_name(stem.name + (".wav" if fmt.startswith("wav_") else ".mp3"))
    out.write_bytes(raw)
    return out


def write_sidecar(master: Path, meta: dict) -> None:
    meta["file"] = master.name
    meta["created"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    master.with_suffix(".json").write_text(json.dumps(meta, indent=2) + "\n")
    print(f"{master}  ({meta.get('headers', {}).get('character-cost', '?')} credits)")


def kept(headers: dict) -> dict:
    return {k: headers[k] for k in KEPT_HEADERS if k in headers}


def sfx(args):
    body = {"text": args.prompt, "model_id": SFX_MODEL, "prompt_influence": args.influence,
            "loop": args.loop}
    if args.seconds is not None:
        body["duration_seconds"] = args.seconds
    for take in range(1, args.takes + 1):
        stem = args.out if args.takes == 1 else args.out.with_name(f"{args.out.name}-{take}")
        started = time.time()
        fmt, headers, raw = call_ladder("/v1/sound-generation", body, PCM_LADDER)
        # Sound effects come back as interleaved 16-bit stereo PCM.
        master = write_master(stem, fmt, raw, channels=2)
        write_sidecar(master, {"endpoint": "/v1/sound-generation", "request": body,
                               "output_format": fmt, "headers": kept(headers),
                               "elapsed_s": round(time.time() - started, 1)})


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="command", required=True)
    p = sub.add_parser("sfx")
    p.add_argument("prompt")
    p.add_argument("--out", type=Path, required=True)
    p.add_argument("--seconds", type=float)
    p.add_argument("--influence", type=float, default=0.3)
    p.add_argument("--loop", action="store_true")
    p.add_argument("--takes", type=int, default=1)
    p.set_defaults(run=sfx)
    args = ap.parse_args()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.run(args)


if __name__ == "__main__":
    main()
