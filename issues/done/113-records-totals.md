# [113] Records: lifetime totals and the next unlock

**Status:** done
**Milestone:** M8
**Depends on:** [111]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
The save counts runs, claims, kills, levels and play time, but no screen shows them. The
unlocks for games played ([114]) work only if the player sees how close the next one is.

## Approach
- The RECORDS screen ([111]) gets the lifetime totals: games, claims, kills, levels
  cleared, play time, best score, best level.
- Under them, the next unlock and its progress, for example
  "NEXT: GRID FILL AT 25 GAMES (18 / 25)".

## Verification
`cargo test` passes with a test for the next-unlock text; the `records` shot shows the
totals.

## Notes
- From the 2026-11-05 brainstorm (D39).
- Done 2026-11-05. One page: the totals in two columns (BEST and BEST LEVEL, GAMES and
  CLAIMS, KILLS and LEVELS, then TIME), "NEXT: HAZARD WALLS (30 / 50 GAMES)"
  (`skins::next_by_games`: the nearest games tier; no line once all are open), then the
  medals, now 46 units apart (names at 20 px). The skins screen shows the level unlocks,
  so this line shows only the games ones.
- Evidence: `cargo test` 158 passed (totals and play time, the next-unlock text, the
  nearest tier, the layout fit); clippy clean with and without `dev_tools`; `just replay`
  unchanged; `just shot` md5 e170997305a806320c6f9f967cf3cf5b; the `records` shot with a
  full save, looked at.
