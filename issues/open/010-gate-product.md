# [010] Gate: Product decisions for v1

**Status:** open
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

## Problem
Store work (M5, M6) and some code (orientation, playfield shape, persistence, web build)
need owner answers. Only Pedro can give them.

## Approach
Questions for Pedro:
1. Name: is "Rix" final? A store name check is needed (App Store, Play, Steam).
2. Bundle / application id: keep `org.pp.rix` (from `feat/ios` and `feat/android`) or a
   domain Pedro owns (e.g. `me.pramos.rix`)? It cannot change after the first upload.
3. Price model: paid up front, free, free with one unlock, or ads? (Each changes M5 work:
   IAP, ad SDKs, data-safety forms.)
4. Orientation: portrait only (feat/ios Info.plist says portrait), or both?
5. Playfield shape: one fixed aspect on every device (letterboxed; same seed = same game,
   fair to share) or fill the screen (different game per device)? Feeds [019].
6. OS floors: minimum iOS and Android versions (feat/android had minSdk 31).
7. Web build on itch.io: yes or no? (Cargo.toml already has a `wasm-release` profile.)
8. Languages for v1: English only?

## Verification
Every question has a dated answer in `decisions.md`, and this gate closes.

## Notes
- **gate**: issues that depend on it wait, even in auto mode.
