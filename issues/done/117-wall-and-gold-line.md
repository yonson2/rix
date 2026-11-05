# [117] Two more skins: a wall and the GOLD line

**Status:** done
**Milestone:** M8
**Depends on:** [078], [114]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
The unlock tiers ([114]) need a new wall (50 games) and a top reward (1000 games).

## Approach
- One wall skin, drawn in code as the brick and circuit textures are (`src/game/skins.rs`).
- The GOLD line: a glowing gold line, for 1000 games.
- Look only ([067]).

## Verification
`cargo test` passes; `just shot` with each skin (`RIX_SKINS`) shows it; look at the PNGs.

## Notes
- From the 2026-11-05 brainstorm (D39).
- 2026-11-05: [114] came first; add the wall as `Unlock::Games(50)` and GOLD as `Unlock::Games(1000)` (`src/game/skins.rs`), with a test in `looks_unlock_at_5_25_100_250_and_500_games`.
- Done 2026-11-05. HAZARD walls (`Unlock::Games(50)`): yellow and dark diagonal stripes, a
  16 x 16 tile like BRICK and CIRCUIT. GOLD line (`Unlock::Games(1000)`): a soft gold glow
  strip (16 units, it pulses), a pale core and `Glint` marks every 38 units that twinkle
  (`twinkle`: mostly small, a flash now and then).
- Dead ends: stripes 4 cells wide had many diagonal steps, each with red and green CRT
  fringes; 8 cells read as hazard stripes. A thin gold line (no glow, then a 10-unit glow)
  split into red and green under the CRT pass (it shifts red and green about 2.5 px on an
  iPhone); the 16-unit glow keeps it gold.
- Evidence: `cargo test` 156 passed (the tiers with HAZARD at 50 and GOLD at 1000, the
  tile rows, the twinkle range); clippy clean with and without `dev_tools`; `just replay`
  unchanged; `just shot` md5 e170997305a806320c6f9f967cf3cf5b; store-scene shot (seed 4,
  level 6, line set 1, 1179 x 2556) with `RIX_SKINS=gold,hazard`, looked at beside NEON
  and LASER.
