# [002] ARCHITECTURE.md - bird's-eye view, code map, invariants

**Status:** open
**Milestone:** M0
**Depends on:** [003]
**Created:** 2026-11-02
**Closed:** -

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
