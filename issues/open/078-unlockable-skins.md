# [078] Unlockable skins for the line and the walls

**Status:** open
**Milestone:** M4
**Depends on:** [036], [040], [048], [070]
**Created:** 2026-11-03
**Closed:** -

## Problem
Pedro's idea (2026-11-03, D20): the player unlocks skins by reaching a level or a score.
A skin changes the look of the line (for example a chain instead of the neon line) or of
the walls ("cool things"). Today the line is cyan sprite segments
(`polygon::handle_draw_line`) and the walls are gray meshes with a `ColorMaterial` that
glow while a line is drawn (`wall.rs`, `WallEvent`).

## Approach
- A `LineSkin` and a `WallSkin` enum; the default skins are today's look.
- An unlock rule per skin (best level or best score), checked at game over; unlocked and
  chosen skins live in the saved data ([048]). The first unlock shows a short notice,
  as the tips of [070].
- A skins screen from the title menu ([036]) to pick one line skin and one wall skin;
  a locked skin shows its rule.
- Look only: a skin never changes the collider, the line thickness or the gameplay, so a
  seed plays the same with any skin ([067]).
- Line skins draw the segments with their own mesh or texture (a chain: repeated link
  sprites along the segment); wall skins use a material (a shader, as the
  `effects` materials) or a PixelLab texture ([011] Q5). Each skin keeps the neon look
  with bloom and CRT ([011] Q1).

## Verification
A unit test of the unlock rules (level and score thresholds); a fresh save has only the
default skins; `just shot` with each skin forced (a dev env var) shows the line and the
walls in that skin.

## Notes
- Pedro, 2026-11-03: "we could add certain skins that the player will unlock when
  reaching certain levels or scores (for the walls and for the line, something like
  chains for the lines (dunno, just thinking out loud here) and cool things for the
  walls".
- Open questions for Pedro (answer before the build starts): how many skins in v1, and
  which ones; the unlock thresholds; whether the Star item's rainbow line ([069]) wins
  over a line skin while it lasts (proposal: yes).
