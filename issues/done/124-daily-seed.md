# [124] Daily seed

**Status:** done
**Milestone:** M9
**Depends on:** [109], [123]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
A seed for each day gives every player the same field that day, and a reason to come back
(see [109] Q1). [012] Q4 said no daily seed for v1; D39 opens the question for 1.3.

## Approach
[109] Q1: yes.
- The day's code comes from the UTC date: the same for every player, with no server.
- It sits at the top of the CHALLENGES screen and is open from the start: the day's seed
  with one of the curated rule sets, in turn (a daily challenge), also the ones that the
  list still locks.
- `Save` keeps the best of the day and the days in a row.

## Verification
`cargo test` passes with tests for the date to code (the same all day, a new one the next
day) and the days-in-a-row rules; the `challenges` shot shows the daily entry.

## Notes
- From the 2026-11-05 brainstorm (D39); [109] Q1: yes.
- Result: in `src/game/challenges.rs`: `today` (whole UTC days since 1970), `daily(day)`
  (`CHALLENGES[day % 10]`: each rule set in turn, the locked ones too) and
  `daily_seed(day)` (that rule set on a field from `fastrand::Rng::with_seed(day)`).
  `name` says "DAILY BARE HANDS" for today's code (PLAY SEED, game over, share); the
  code of an older day is a custom challenge. The CHALLENGES screen's first row is the
  daily, always open, with "STREAK N" (the days in a row) under its best; the rows moved
  to a 56 gap (title 340, BACK -345) to make room for the eleventh.
- `Save::daily` (`Daily { day, streak }`): `record_daily` (after `record_run`) counts a
  run of today's daily once per day: the next day goes on the row, a missed day starts
  again at 1 (`Daily::streak_on` shows 0 once a day is missed). The best of the day is
  the daily code's entry in `challenge_bests`; `record_daily` drops the entry of the day
  played before, so the save keeps one. A daily started before midnight and ended after
  it does not count (its code is then no longer today's).
- New direct dependency `web-time` 1.1.0 (already in the tree through Bevy, so the lock
  and `assets/licenses/third-party.md` keep the same crates): `std::time::SystemTime`
  panics on the web. A check for the web build is in the [057] notes.
- Evidence: `cargo test` 198 passed; new tests: `a_day_has_one_daily_code_and_the_next_day_another`
  (the same code all day, another the next day, each rule set in ten days, the names),
  `the_daily_counts_days_in_a_row_and_keeps_the_best_of_the_day`,
  `the_daily_comes_first_open_with_its_days_in_a_row`. `RIX_SHOT_SCENE=challenges` with
  yesterday played 3 days in a row: "DAILY BARE HANDS", "NO ITEMS", "STREAK 3" on top.
  `RIX_SEED` at today's daily seed with `RIX_CLOCK=1`: the game over says "DAILY BARE
  HANDS" and the save goes to day 20762, streak 4. `just replay` passes; default
  `just shot` unchanged (md5 e170997305a806320c6f9f967cf3cf5b); fmt, both clippy runs
  and `cargo build` clean.
