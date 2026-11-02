# [066] rustfmt and clippy clean, so CI can enforce them

**Status:** done
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
CI ([008]) is to run `cargo fmt --check` and clippy with `-D warnings`. Both fail today on
old code: rustfmt on files from before [006], clippy with 9 warnings (7 `collapsible_if`,
2 `unnecessary_unwrap`). Pedro (2026-11-02): clean this up before CI.

## Approach
One `cargo fmt` over the crate (format only), then fix the 9 clippy warnings without a
change in behaviour. The dead code and the lint table stay in [035].

## Verification
`cargo fmt --check` passes; `cargo clippy --all-targets -- -D warnings` and
`cargo clippy --all-targets --features dev_tools -- -D warnings` pass; `cargo test` passes;
`just shot` and `just shot-menu` are pixel-identical to the [064] shots.

## Notes
- Split from [035] (its fmt note) because [035] waits on [026].
- Done: `cargo fmt` over the crate (`level.rs`, `polygon.rs`, `wall.rs`; rustfmt first
  needed two trailing spaces removed by hand in `wall.rs`). Clippy: the 7
  `collapsible_if` became let-chains (`cargo clippy --fix`, same short-circuit order); the
  `unnecessary_unwrap` sites in `handle_input` became `if let Some((side, edge)) =
  maybe_edge && ..` (3 sites: fixing 2 exposed the third).
- Checks: `cargo fmt --check` passes; clippy with `-D warnings` passes with and without
  `dev_tools`; `cargo build` has no warning; `cargo test` (32) passes. Shots: game, menu
  and seed 4 are pixel-identical (AE 0) to shots of the code before this change.
  (`target/` was cleaned outside this session at 16:56, so the [064] PNGs were gone; the
  baseline came from the committed code, built with this change stashed.)
- AGENTS.md: rustfmt and clippy `-D warnings` are now rules, not "no new warnings".
