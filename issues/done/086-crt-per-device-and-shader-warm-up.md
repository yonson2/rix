# [086] CRT per device and shader warm-up

**Status:** done
**Milestone:** M4
**Depends on:** [025], [082]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Split out of [082] (2026-11-03): the CRT pass uses one set of `PostProcessSettings` on
every screen, and the custom pipelines (CRT, explosions, glow) compile on first use, which
may hitch on a phone. Both need [025]'s device numbers first: today nobody knows whether
the scanlines or the grid look wrong on a phone, or whether the first kill or bomb hitches.

## Approach
- From [025]'s device notes: if the scanlines or the grid look wrong on a phone, tune
  `PostProcessSettings` per device class (pixel density or screen size).
- If the first kill or bomb hitches, spawn one hidden-but-drawn entity per material on
  `Screen::Loading`, so the pipelines compile before play.
- If neither shows on a phone, close this issue with that finding.

## Verification
[025]'s phones show no CRT artefact and no hitch on the first kill or bomb (frame-time log,
[024]); `just shot` and `cargo test` pass.

## Notes
- Closed 2026-11-03 (D34): Pedro played on his Android phone; on the CRT and the hitch on
  the first kill or bomb: "good". No per-device CRT settings and no shader warm-up are
  needed. No frame-time log was taken: Pedro's run is the check.
