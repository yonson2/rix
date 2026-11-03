# [058] Web release build and CI release workflow

**Status:** open
**Milestone:** M6
**Depends on:** [008], [057]
**Created:** 2026-11-02
**Closed:** -

## Problem
No web build exists, and nothing builds a release.

## Approach
A tag-triggered GitHub Actions release workflow (bevy_new_2d's release workflow as the
base, [026]) that builds only the web target: `wasm32-unknown-unknown` in the
`wasm-release` profile, `wasm-bindgen`, `wasm-opt`, and a zip of the page and assets. No
desktop builds (D13).

## Verification
A test tag produces the web zip; served from a local HTTP server it loads in a desktop
and a phone browser and plays a run.

## Notes
- [010] Q7 (2026-11-02): the web build is in. D9: the only desktop store is itch.io. Open
  for Pedro: desktop downloads on itch.io as well, or web only? (macOS downloads need a
  Developer ID for signing and notarization.)
- D13 (2026-11-02, Pedro): web only on itch.io, no desktop downloads.
- From [048] (2026-11-03): the web build keeps no save (`save::data_dir` is `None` on
  wasm): best score, stats and settings reset on reload. Store the same JSON in
  `localStorage` (web-sys `Storage`) here.
- From [075] (2026-11-03): CI checks the replay hashes on Linux x86_64 and macOS arm64 only.
  A wasm check needs a headless wasm runner that loads the assets, and the `RIX_STATE`
  hash writes `usize` values (`kill_count`, the `Vec` lengths) as 4 bytes on 32-bit:
  hash them as `u64` first. Gameplay draws already use `u64` (`pick_kind`, the bomb pick).
