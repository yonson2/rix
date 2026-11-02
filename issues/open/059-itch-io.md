# [059] itch.io page and butler upload

**Status:** open
**Milestone:** M6
**Depends on:** [058], [051]
**Created:** 2026-11-02
**Closed:** -

## Problem
itch.io is the cheapest desktop (and web) store and has no review queue: a good first
desktop release.

## Approach
An itch.io page (owner-run), `butler push` from the release workflow ([058]) with
`BUTLER_CREDENTIALS` as a repo secret, channels per platform, a web channel if [010] Q7
says yes.

## Verification
The itch.io page offers the builds and each downloads and runs.

## Notes
- Page and API key are **owner-run**.
- bevy_new_2d's release workflow already pushes to itch.io with butler, one channel per
  platform (`docs/research/bevy_new_2d.md`). For web it sets `AssetMetaCheck::Never` and
  `fit_canvas_to_parent`. Rix web risks: `src/crt.rs` hardcodes `Rgba16Float` and the WebGL2
  uniform padding is commented out.
- D9 (2026-11-02): itch.io is the only store besides the mobile stores; the web channel is in ([010] Q7).
