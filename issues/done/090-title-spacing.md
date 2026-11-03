# [090] Title menu: more space between the buttons

**Status:** done
**Milestone:** M4
**Depends on:** [036], [078]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro (D27): "I want a tiny bit more spacing between items in the main menu." Today the
buttons are 76 high, 84 apart (8 units of space), and WIKI and SKINS are 6 apart.

## Approach
- Buttons 90 apart (14 units of space): PLAY 0, PLAY SEED -90, SETTINGS -180, WIKI and
  SKINS -270, QUIT -360 (desktop only; its bottom stays inside the 800-high view).
- WIKI and SKINS 14 apart too: each 123 wide at x = +-68.5 (the row stays 260 wide).

## Verification
`just shot-menu` shows even gaps and QUIT inside the view; the emulator shows the title.

## Notes
- Done 2026-11-03 as planned: buttons 90 apart (PLAY 0 to QUIT -360), WIKI and SKINS 123
  wide at x = +-68.5.
- Evidence: `just shot-menu` (even gaps, QUIT inside the view); the emulator title.
