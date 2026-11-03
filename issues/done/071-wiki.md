# [071] Wiki - enemies and items explained

**Status:** done
**Milestone:** M4
**Depends on:** [036], [070]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
Pedro's idea (2026-11-02, D18): a wiki section in the menu that explains each enemy and
each item, so a player can look up what something does.

## Approach
A menu entry ([036]) that opens a list: one card per enemy and item with its look (the
live shape or sprite), its name and one or two lines on what it does. Kinds not yet met
([070]) show as locked ("???"). The text lives next to the spawn data ([032]) so one
place names and describes each kind.

## Verification
The menu opens the wiki; every enemy and item kind in the spawn table has a card; a kind
not met yet shows locked (`just shot-menu` with a wiki scene, and a test that every kind
has a description).

## Notes

- 2026-11-03, done. `screens/wiki.rs` (`Screen::Wiki`): title WIKI, one row per kind in
  `tutorial::KINDS` (now public: `Kind::id`/`name`/`about`), BACK to the title. A row has
  the kind's look, its name and what it does; a kind not in `Save::met` shows "?", "???"
  and "NOT MET YET". The looks are functions in each kind's module (`ball::look`,
  `square::look`, `arrow::look`, `splitter::look`, `shooter::look`, `basic_bomb::look`,
  `items::token`); spawn uses them too, except the ball's and the square's (they grow and
  fade in play). The bomb's 100-wide glow mesh shows at half size.
- Title: WIKI under SETTINGS; the buttons are now 84 apart (PLAY -10 ... QUIT -346) so
  QUIT still fits the 800-tall desktop view; the logo and BEST moved up 30 and 25.
- `RIX_SHOT_SCENE=wiki` (dev_tools) opens the wiki and shoots it at the menu's time.
- Verification: tests `every_spawned_kind_has_a_row_with_a_name_and_a_text` (every kind
  of `SPAWN_TABLE` and `ITEM_LEVELS` has a row, each text fits) and
  `a_kind_not_met_shows_locked`; wiki shots with a save that met five kinds (four locked
  rows) and without a save (all nine met); `just shot-menu` with WIKI (md5 changes, the
  title moved); emulator: title with WIKI, a tap opens the wiki, BACK returns to the title (rows moved 40 right after the phone picture). 105 tests pass, clippy
  clean with and without `dev_tools`, `just replay` unchanged, `just shot` md5 unchanged
  (e170997305a806320c6f9f967cf3cf5b).
