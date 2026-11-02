# [007] Unit tests for the pure gameplay logic

**Status:** open
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

## Problem
There are no tests. The geometry (point in polygon, polygon completion around the walls,
choice of the side to claim, self-intersection of the drawn path), the score formatting and
the enemy count curve are pure functions that the upgrade and the M3 rework will touch.

## Approach
In-module `#[cfg(test)]` tests for the pure functions in `src/rix/polygon.rs`,
`src/rix/user_input.rs`, `src/rix/wall.rs`, `src/rix/setup.rs` (`Points` display) and
`src/rix/enemies/rng.rs`. Pin current behaviour, including known quirks (note them in the
test name), so the upgrade cannot change them silently.

## Verification
`cargo test` passes and covers each function named above.

## Notes
