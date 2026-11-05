# [127] Menu UI overhaul

**Status:** done
**Milestone:** none (an experiment on `exp/menu-ui`, merged as 1.3.1, D41)
**Depends on:** [036], [123], [124]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
Pedro (2026-11-05): "Right now, while looking ok, it feels a bit cluttered. Think long and
hard on how to make a better ui and try to do it, then do comparison screenshots and
create an artifact to showcase it to me."

The title shows nine equal boxes in one style (PLAY, CHALLENGES, PLAY SEED, WIKI, SKINS,
RECORDS, SETTINGS, CREDITS, QUIT), so nothing leads the eye. The labels shrink to fit
their boxes, so one grid shows text from 16 px (CHALLENGES) to 34 px (WIKI). Play modes,
collections and utilities share one grid. The settings rows put the value in the label,
so a row reads as a command, not as a setting with a state.

## Approach
One idea: a hierarchy by purpose. Each tier has its own look and the same text size.
- **Primary** (filled, large, a slow glow): PLAY on the title, RETRY on the game over
  screen, RESUME in the pause menu.
- **Secondary** (outlined): CHALLENGES, now a card that also shows the day's challenge
  and the streak.
- **Collection tiles** (icon and label, one size): SKINS, RECORDS, WIKI.
- **Utility at the edges:** a gear for SETTINGS in the top corner; CREDITS (and QUIT on
  desktop) as plain text at the bottom.
- PLAY SEED moves into the CHALLENGES screen (a code is a challenge too): the CHALLENGES
  screen ends with PLAY SEED and BACK, and the seed screen's BACK goes back there.
- Settings become list rows: the name on the left, the value on the right (ON in green,
  OFF in gray, percent in yellow); a locked MUSIC says when it opens.
- The game over screen: RETRY primary, MENU and SHARE in one row under it.

## Verification
`cargo test`, both clippy runs and `cargo fmt --check` pass; the default `just shot` md5
stays e170997305a806320c6f9f967cf3cf5b; before and after shots (390 x 844) of the title,
challenges, settings and game over screens are in an artifact for Pedro.

## Notes
- Experimental: Pedro decides after the artifact whether it merges.
- 2026-11-05: done on `exp/menu-ui`. The CHALLENGES screen also hides the locked
  challenges after the next one behind "+N MORE TO UNLOCK": a new player saw six gray
  rows. Secondary rows (PLAY SEED and BACK, MENU and SHARE) are 64 high, so their labels
  stay under the primary's 44 px. `locked_button` and its cross line ([091]) are gone:
  the only locked button was MUSIC, now a gray row.
- Evidence: 200 tests pass; both clippy runs, the wasm clippy run and `cargo fmt --check`
  pass; the `just shot` md5 stays e170997305a806320c6f9f967cf3cf5b; `just replay`
  passes; `just shot-flow` still taps RETRY. Shots at 390 x 844 (and the title at
  450 x 800) are in the artifact.
- Not shot: the pause menu (no shot scene for it) and the phone footer (CREDITS alone,
  centered; a desktop shot shows QUIT beside it).
- Result (2026-11-05): Pedro saw the artifact (https://claude.ai/artifact/HAuJJ14FqFcX7pUYxrUgta)
  and said "i like all of your changes": it merges, the gear stays, the locked challenges
  stay hidden (D41).
