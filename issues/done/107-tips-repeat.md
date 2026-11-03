# [107] An item's tip shows again in later levels and runs

**Status:** done
**Milestone:** M5
**Depends on:** [070]
**Created:** 2026-11-04
**Closed:** 2026-11-04

## Problem
Pedro, on the 1.0.0 TestFlight build: "the tutorial for the items I _think_ got duplicated
for a couple of runs for some items". A kind counted as met only when its tip ran its full
3.5 s. A pause, a death or a level clear (each ends `IsPaused::Running`) removed the tip
first, so the kind stayed unmet and its tip came back on the next level or run. D23 says
a tip shows "once (saved)"; the show-again rule was [070]'s own choice.

## Approach
`start_tip` pushes the kind into `Save::met` when the tip shows; `follow_tip` only ends it.
A pause, a death or a level clear ends a tip early, and it does not come back.

## Verification
`cargo test tutorial` passes, with `a_pause_ends_a_tip_and_it_does_not_show_again`.

## Notes
- 2026-11-04: Verification passes. `cargo test` 134 passed; clippy clean with and without
  `dev_tools`; `just replay` hashes unchanged; `just shot` md5 unchanged
  (e170997305a806320c6f9f967cf3cf5b). Found by Pedro on the 1.0.0 TestFlight build; ships in 1.0.1.
