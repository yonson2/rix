# [089] Soundtrack unlock: sound effects only at first, music as a reward

**Status:** done
**Milestone:** M4
**Depends on:** [043], [048], [078]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
D26 (Pedro, 2026-11-03): a new player hears only the sound effects. The soundtrack (menu,
game and Star music) unlocks later, as one more reward, and then stays off until the player
turns it on. Today the music plays from the first start at 60 % (`Settings::default`).

## Approach
- `SOUNDTRACK_LEVEL` in `save.rs`: the best level ([078], `Save::best_level`) that unlocks
  the soundtrack. Proposal: 3, so the first reward comes before the first skin (level 5).
- `Settings::default` sets the music to 0 %.
- While the soundtrack is locked, the MUSIC row in the settings (and the pause panel, [073])
  reads "MUSIC: LEVEL 3" and a tap does nothing, and `apply_settings` sets the music volume
  to 0 whatever the save holds.
- The level start that unlocks it shows the [078] notice: "SOUNDTRACK UNLOCKED / TURN IT ON
  IN SETTINGS". After that the row shows "MUSIC 0%"; a tap steps it to 100 % ([048]).

## Verification
Unit tests: a fresh save plays no music and its MUSIC row is locked; at the unlock level the
row opens at 0 % and a tap sets 100 %; the level start that unlocks it shows the notice once.
`cargo test`, clippy and `just replay` pass (the replay hashes do not move); a
`just shot-menu` and an emulator look at the settings screen.

## Notes
- Open question for Pedro: the unlock level (proposal 3, in place). It is one constant,
  `save::SOUNDTRACK_LEVEL`; the question waits in [047] (balance pass).
- Done 2026-11-03. `save::SOUNDTRACK_LEVEL` (3) and `Save::soundtrack_open`;
  `Settings::default` music 0 %; `apply_settings` keeps the music volume at 0 while locked;
  the MUSIC row (settings screen and pause panel) reads "MUSIC: LEVEL 3" and does not step;
  `skins::unlock_notice` shows "SOUNDTRACK UNLOCKED / TURN IT ON IN SETTINGS" at the level
  start that opens it (the same notice as a new skin).
- An old save (Pedro's phones) keeps its stored music percent: once it reaches level 3,
  the music plays at that percent, not at 0. A new install starts at 0 %.
- Evidence: tests `the_music_plays_only_once_the_soundtrack_is_open_and_turned_on`,
  `music_is_locked_until_the_soundtrack_opens_then_starts_off`,
  `the_soundtrack_unlocks_once_with_a_notice`; 121 tests; clippy clean with and without
  `dev_tools`; `just replay` unchanged; on the emulator (fresh save) the settings show
  "MUSIC: LEVEL 3" and a tap leaves it so.
