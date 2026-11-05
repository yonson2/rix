# [118] Kill counts in the wiki

**Status:** done
**Milestone:** M8
**Depends on:** [071]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
The wiki shows each enemy kind, but not how many of each the player has eaten. A count
gives the wiki a reason to come back to.

## Approach
- `Stats` counts the kills per kind, added at the end of a run as `kills` is.
- Each enemy page in the wiki shows "EATEN: 1,204", with a mark at 100 and at 1000.

## Verification
`cargo test` passes with a test for the count per kind; a `wiki` shot shows the count.

## Notes
- From the 2026-11-05 brainstorm (D39).
- Done 2026-11-05. `on_despawn_enemy` counts a kill in `Run::eaten` by kind id (`Kind::id`:
  "Ball", "Square", ...; a kill with no kind counts only in the total); `record_run` adds
  it to `Stats::eaten` (`Stats` is no longer `Copy`). The wiki row: "EATEN n" at the right
  of the name, gray under 100, yellow with one star (the STAR icon at 0.45) from 100 and
  two from 1000.
- Dead end: the mark as "*": this font draws it as a small dot; it has no star glyph.
- Evidence: `cargo test` 160 passed (a kill counts for its kind, `record_run` adds the
  counts, the marks); clippy clean with and without `dev_tools`; `just replay` unchanged;
  `just shot` md5 e170997305a806320c6f9f967cf3cf5b; a `wiki` shot with 1,204, 120, 33, 0
  and 7, looked at.
