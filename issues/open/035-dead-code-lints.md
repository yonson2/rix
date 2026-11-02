# [035] Dead code, lint table and clippy clean

**Status:** open
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** -

## Problem
Unused (2026-11-02, by grep): `generate_item_position` (`utils.rs`),
`get_window_inner_corners` (`wall.rs`), `get_cursor_world_position` (`camera.rs`), `area`
and `PathSegment` (`polygon.rs`), `PlayableArea` (written, never read), and
`PhysicsPickingPlugin` (no picking observers exist). `Cargo.toml` has a commented-out lints
table. Several `#[allow(clippy::...)]` hide queries that could be `QueryData` or
`SystemParam` types.

## Approach
Delete what stays unused after M1 and [026]; enable a reviewed lint table in `Cargo.toml`
(bevy_new_2d's as a start, [026]); fix or justify each allow.

## Verification
`cargo clippy --all-targets -- -D warnings` passes with the lint table on; `cargo build`
reports no dead-code warnings.

## Notes
