# [031] Score overflow at high levels

**Status:** open
**Milestone:** M3
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

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
