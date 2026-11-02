# [044] New enemy - Splitter

**Status:** open
**Milestone:** M4
**Depends on:** [032], [012]
**Created:** 2026-11-02
**Closed:** -

## Problem
Three enemy kinds (Ball, Square, Arrow) give little variety. todo.md: "The Splitter - When
hit or after a set time, splits into two smaller, faster versions of itself."

## Approach
Pedro's untracked `splitter.md` sketches one design: a "Queen" octagon that, when claimed,
spawns two faster "Soldiers"; straight-line movement with random turns. Confirm the design
in [012], then build it on the spawn table ([032]).

## Verification
A seeded run spawns a Splitter; claiming it spawns two soldiers (test or scripted shot).

## Notes
- todo.md: Splitter line above. `splitter.md` (untracked, D8) holds a step list written for
  the old code structure.
