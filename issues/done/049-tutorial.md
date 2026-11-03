# [049] First-run tutorial

**Status:** done
**Milestone:** M4
**Depends on:** [036], [037]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
A new player does not know the rules: start a line on a wall, reach another wall, the side
without enemies is claimed, an enemy touching the line kills you, claim 75% to clear.

## Approach
A short guided first level: a ghost finger shows the drag, then one line of text per rule
as it first happens. Skippable; shown once (flag in [048]).

## Verification
A fresh install shows the tutorial once; a second start skips it.

## Notes
- Done 2026-11-03 (`src/game/tutorial.rs`). On level 1 while `Save::tutorial_done` is
  false: a ghost finger drags from the left wall to the right wall with "DRAG A LINE FROM
  WALL TO WALL"; the first line start shows "AN ENEMY THAT TOUCHES THE LINE KILLS YOU";
  the first claim shows "THE SIDE WITHOUT ENEMIES IS YOURS", then after 3 s "CLAIM 75% TO
  CLEAR THE LEVEL" (from `LevelProgress::target`), then it ends. SKIP (top right of the
  field) ends it at once.
- My choices (Pedro can change them): the game does not stop for it, so a seed replays the
  same with or without it; the first claim (or SKIP) sets the flag, so a player who dies
  before a claim sees it again; an old save file (no flag) shows it once; a pause hides it
  and the resume shows it again (its SKIP would otherwise sit under the pause menu
  buttons). A shot run without `RIX_SAVE` skips it, so `just shot` stays the same.
- Evidence: `cargo test` 98 passed (step order, finger path, an app test: a fresh save shows
  it, a pause hides it, the resume shows it, SKIP ends it for good; a done save or level 2
  shows nothing; an old save reads `tutorial_done` false); clippy clean with and without
  `dev_tools`; `just replay` unchanged; `just shot` md5 unchanged
  (e170997305a806320c6f9f967cf3cf5b). Verification: two `just shot` runs with one fresh
  `RIX_SAVE` file: the first shows the tutorial (claim rule and SKIP in the picture) and
  writes `"tutorial_done": true`; the second gives the baseline picture (same md5). A
  patched run (line delayed; reverted) shows the ghost finger and the first rule at 450 x 800
  and 390 x 600. `just shot-flow` with a fresh save: level 1 has the overlay (+6 entities,
  +2 texts), the level clear removes it, level 2 and the retry show none.
