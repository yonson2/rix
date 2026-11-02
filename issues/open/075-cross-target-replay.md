# [075] Replay without the renderer, checked on two targets in CI

**Status:** open
**Milestone:** M4
**Depends on:** [072]
**Created:** 2026-11-03
**Closed:** -

## Problem
`just replay` ([072]) shows that a seed gives the same state at any frame rate on one
machine. Seed sharing ([067]) needs the same state on every target: iOS and Android
(arm64), the web build (wasm on x86 and arm hosts). avian's `enhanced-determinism` is on,
but no run compares two targets. The replay runs the full shot harness, which renders, so
CI (no GPU) cannot run it.

## Approach
A replay mode with no render plugins (or a headless app test with the game plugins and
stub assets) that logs `RIX_STATE`; a CI job runs it on `ubuntu-latest` (x86_64) and
`macos-latest` (arm64) and compares the hashes. Add wasm if it runs under a headless
runner cheaply.

## Verification
The CI job passes with the same `RIX_STATE` hash for seeds 1, 2, 5, 7 and 8 on both
runners.

## Notes
- From [072] (2026-11-03): cross-target was "if cheap" there; it is not, because the
  replay renders.
