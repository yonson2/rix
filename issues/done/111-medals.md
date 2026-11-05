# [111] Medals and the RECORDS screen

**Status:** done
**Milestone:** M7
**Depends on:** [048], [070]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
D39: medals come in 1.1, with the leaderboards. Feats that only Rix has give goals past
the best score.

## Approach
- The medals as data (id, name, rule text). `Save.medals` holds the earned ids, as `met`
  does. A system checks each feat at its moment; a new medal shows a short notice, as a
  skin unlock does, once.
- A RECORDS button on the title opens the RECORDS screen: every medal, earned or not, with
  its rule text. [113] adds the lifetime totals to the same screen.
- No medal in the first run (D27: the first run unlocks nothing). Challenge runs earn
  medals too ([109] Q3).
- The first list:
  - ONE STROKE: clear a level with one claim.
  - FEAST: eat 3 enemies with one claim.
  - PACIFIST: clear a level with no kills.
  - STAR STRIKE: kill 5 enemies during one Star.
  - BOMB SQUAD: bomb 3 enemies in one level (a bomb takes one enemy, so "3 with one bomb"
    cannot happen).
  - LAND GRAB: clear a level at 90 % or more (renamed: the challenge GREEDY, [123], keeps
    the name).
  - KNOW YOUR ENEMIES: meet every enemy kind.
  - DEEP DIVE: reach level 25.

## Verification
`cargo test` passes with a test per medal (earned at its feat, once, saved); a `records`
shot scene (`RIX_SHOT_SCENE=records`) shows the screen with earned and open medals.

## Notes
- From the 2026-11-05 brainstorm (D39: "ok" to medals in 1.1).
- [112] makes each medal an achievement on Game Center and Play Games.
- Done 2026-11-05. `src/game/medals.rs`: `Medal` (id, name, rule), the `Earn` event and
  `on_earn` (not in the first run, once, a `MEDAL: NAME` notice above the skin notice; two at
  once stack). `LevelTally` counts a level's claims, kills, bombs and Star kills: the level
  clear checks ONE STROKE, PACIFIST and LAND GRAB; `polygon` triggers FEAST, `basic_bomb`
  BOMB SQUAD, the `DespawnEnemy` observer STAR STRIKE; DEEP DIVE at the level start, KNOW
  YOUR ENEMIES when `Save::met` has every enemy. `Save::medals` holds the ids.
- `src/screens/records.rs`: RECORDS, MEDALS n / 8, every medal with its rule (earned:
  yellow and white, open: gray), BACK. The title: [WIKI | SKINS], [RECORDS | SETTINGS],
  then CREDITS (and QUIT on desktop).
- Evidence: `cargo test` 145 passed (tests for the clear medals, BOMB SQUAD, every enemy,
  unique ids, earned once with notices and none in the first run, STAR STRIKE); clippy
  clean with and without `dev_tools`; `just replay` unchanged; `just shot` md5
  e170997305a806320c6f9f967cf3cf5b (unchanged); `RIX_SHOT_SCENE=records` with a save of 3
  medals shows them lit and the other 5 gray.
