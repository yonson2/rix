# [069] New item - Star (rainbow line, invulnerable)

**Status:** done
**Milestone:** M4
**Depends on:** [033], [041], [043]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- 2026-11-03, done (the music swap moved to [088], it waits for Pedro's pick, D25).
  `items/star.rs`: `Item::Star` from level 5 (`ITEM_LEVELS`; level 1 replays stay the
  same), a white star token. Catching it inserts the `Star` resource for `STAR_SECS` (6 s,
  [047] tunes it); `handle_die` ignores `Die` while it exists, so neither an enemy nor a
  bullet on the line kills. The effects label shows "STAR 6". `rainbow_line` colors each
  line segment by time and position (hue 360 degrees per second and 0.8 per world unit,
  1.8 glow, so it blooms).
- Decision (what an enemy does on the line): it passes through. The line has colliders but
  no body, so avian 0.8 reports the touch but pushes nothing (test
  `a_ball_passes_through_the_line_and_touches_it`). A claim then decides the sides as
  always. A claim that ends after the timer is not special: the next new touch kills.
- Pickup sound: ElevenLabs, three takes, `star-3` picked by measurement (several note
  onsets, the "rising arpeggio" of the prompt; the others are single decaying hits),
  peak -3 dB as the other item pickups, `Sfx::Star`, no haptic pulse.
- Tip and wiki: `Kind::Item(Item::Star)` in `tutorial::KINDS` ("TOUCH THE STAR:" /
  "INVINCIBLE FOR 6 S"); the wiki shows ten rows (row gap 58 so BACK stays clear).
- Verification: tests `a_touch_kills_only_without_the_star` (with `Star` a `Die` changes
  nothing; without it the state goes to `Dying`), `a_ball_passes_through_the_line_and_touches_it`,
  `the_rainbow_moves_along_the_line_and_over_time`, the item activation test (a star
  inserts `Star` and plays `Sfx::Star`), `pick_item` (the star from level 5); a patched shot
  (Star on from the start, the scripted line slowed to 200 ticks, both restored) shows the
  rainbow line with bloom and "STAR 57"; the wiki shot shows the star row. 109 tests pass,
  clippy clean with and without `dev_tools`, `just replay` unchanged, `just shot` md5
  unchanged.

