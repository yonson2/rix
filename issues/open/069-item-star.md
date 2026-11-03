# [069] New item - Star (rainbow line, invulnerable)

**Status:** open
**Milestone:** M4
**Depends on:** [033], [041], [043]
**Created:** 2026-11-02
**Closed:** -

## Problem
Pedro's idea (2026-11-02, D18): an item like the star in Mario Kart. While it lasts, the
line the player draws is an animated rainbow and the player is invulnerable: an enemy
that touches the line does not kill. The music changes while it lasts, as in Mario Kart.

## Approach
An `Item::Star` on the item spawn path ([033]): catching it starts a timer (a few
seconds, tune in [047]). While the timer runs: the death check on line contact is off,
the line material cycles a rainbow (a shader or a per-frame hue shift on the line
meshes), and the audio system ([041]) swaps to a short fast "star" track ([043]), then
back. Decide what an enemy touching the line does in this mode (passes through, or
bounces) and whether a claim that ends after the timer is still safe.

## Verification
A seeded run that catches the Star: an enemy crosses the drawn line during the effect and
the player lives; the line shows the rainbow (`just shot`); the music swaps and swaps
back; after the timer, a touch kills again (test on the death check).

## Notes
- [012] Q1 listed three v1 items ([046]); the Star is a fourth, asked by Pedro on
  2026-11-02.
- From [084] (2026-11-03): the Star's pickup sound is part of this issue. Make it as
  [084] did (`tools/elevenlabs.py sfx`, three takes, a new `Sfx` variant, a row in
  `audio-src/sources.md`).
- From [070] (2026-11-03): the Star's first-sight tip is part of this issue (add the kind to
  [070]'s tip list).
