# [035] Dead code, lint table and clippy clean

**Status:** done
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- Unused but `pub` (no dead-code warning): `polygon::area`, `wall::get_window_inner_corners`
  and its helper `wall::world_to_window_coords` (found in [007]). `PhysicsPickingPlugin`
  (`base.rs`) has no consumer (found by the [014] research).
- The rustfmt and clippy cleanup moved to [066] (done 2026-11-02): clippy is clean, so
  this issue keeps the dead code and the lint table.
- Also unused (found in [002]): `GameLayer::Shape`.
- [019] (2026-11-02): pre-existing dead code that still reads the window: `wall::get_window_inner_corners` (and its `world_to_window_coords`), `utils::generate_item_position`, `camera::get_cursor_world_position`. No callers; remove or port to `FIELD`.
- From [026] (2026-11-02): make the modules private (`mod`, not `pub mod`) where
  nothing outside needs them, so rustc reports the dead code; start the lint table from
  bevy_new_2d's (`docs/research/bevy_new_2d.md`). Paths moved: `src/rix/` is
  `src/game/`, `colors.rs` is `theme/palette.rs`.
- [074] (2026-11-02): `polygon::area` is gone; the shoelace is now `territory::signed_area`
  (used by the claim and progress).
- Done 2026-11-03:
  - `lib.rs`: only `app`, `crt` and `theme` stay public (the examples use
    `crt::PostProcessSettings` and `theme::palette`). With the other modules private,
    rustc listed the dead code.
  - Removed: `camera::get_cursor_world_position`, `wall::get_window_inner_corners` and
    `world_to_window_coords`, `utils::generate_item_position`, `FadeShape::from_seconds`,
    `polygon::PathSegment` (and its census filter), `enemies::TeleportTimer`,
    `GameLayer::Shape`, `PlayableArea` (written, never read), `PhysicsPickingPlugin`
    (nothing uses picking). `Territory` keeps only the marker; nothing read its outline
    since [074]. Removing `GameLayer::Shape` moves the `Item` bit; the layers are used by
    name, and the shots are the same.
  - `[lints.clippy]` from bevy_new_2d: `too_many_arguments` and `type_complexity` allowed;
    the seven per-item `#[allow]`s are gone. Left out of the template table:
    `nonstandard_macro_braces` and its `clippy.toml` (Rix uses no `children!`) and the
    `cfg(bevy_lint)` check (Rix does not run bevy_lint). The old commented table (cast and
    arithmetic lints) is dropped: it was never on.
- Verification 2026-11-03: `cargo build` and `cargo build --features dev_tools` show no
  warning; `cargo clippy --all-targets -- -D warnings` passes with and without
  `dev_tools`; fmt clean; `cargo test` 46 passed; `cargo check -p rix_mobile` clean.
  `just shot` md5 unchanged for seeds 1, 7, 10 and the menu; `just replay` passes;
  `just shot-flow` census unchanged.
