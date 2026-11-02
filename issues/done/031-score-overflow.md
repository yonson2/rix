# [031] Score overflow at high levels

**Status:** done
**Milestone:** M3
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
`handle_update_points` and `handle_increase_points` (`level.rs`) multiply by
`2u128.pow(level - 1)`. `amount * 2^(level-1)` overflows u128 from about level 122 (100 ×
2^121 > u128::MAX) and `pow` itself from level 129: a panic in debug, a wrapped score in
release.

## Approach
Saturating arithmetic (`checked_pow`, `saturating_mul`), or cap the multiplier as part of
the balance pass ([047]).

## Verification
A unit test awards points at level 200 without a panic and with a saturated score.

## Notes
- Done (2026-11-02): `level_points` (`level.rs`) doubles per level with `checked_pow` and
  `saturating_mul`; both point systems add with `saturating_add`. The score stops at
  `u128::MAX` (shown as `3.402e38`). A cap on the multiplier stays a balance question
  ([047]).
- Evidence: tests `points_double_per_level_and_saturate` (50 at level 3 = 200; level 123
  and 200 saturate) and `points_at_level_200_saturate_the_score` (both systems at level
  200 from `u128::MAX - 10`: no panic, `u128::MAX`; the old `2u128.pow(199)` panics).
  `cargo test` 37 passed, clippy clean both ways, fmt clean, seed 1 shot unchanged
  (`109eb09e`).
