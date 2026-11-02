# [002] ARCHITECTURE.md - bird's-eye view, code map, invariants

**Status:** done
**Milestone:** M0
**Depends on:** [003]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
The code flow (input → path → polygon → claimed area → level) and its invariants (what
despawns when, which RNG drives gameplay, which entities collide) live only in the code.

## Approach
Write `ARCHITECTURE.md` in the style of https://architecture.md/ (matklad): a short
bird's-eye view, a code map that names modules and key types (no links, so it does not
rot), the state machine, the main data flow, cross-cutting concerns (rendering, physics,
coordinates, randomness) and the invariants. It describes what IS; the target shape lives in
issues ([026] and M3). Update it in the same commit as any structural change.

## Verification
Every module in `src/` and every shader in `assets/shaders/` appears in the code map, and
every type the doc names exists (`grep` each one).

## Notes
- Done: `ARCHITECTURE.md` at the repo root: bird's-eye view, code map (every module in
  `src/` and every shader), state machine, the data flow of one line, cross-cutting
  concerns (coordinates, physics, rendering, randomness, dev tools) and invariants
  (entity lifetimes, input, territory outlines, RNG, D2).
- Check: a script lists every `src/**/*.rs` module and `assets/shaders/*.wesl` file; all
  appear in the doc. Every identifier in backticks exists in `src/` or `examples/` (the
  only misses are `NEON_*`, a pattern, `bevy_prototype_lyon`, the removed crate, and
  `grep`).
- Found while reading: more dead code (added to [035] Notes) and an area sampler box that
  takes in part of the top wall (added to [029] Notes).
