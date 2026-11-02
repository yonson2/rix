# [001] AGENTS.md - agent working agreements and the game in one page

**Status:** done
**Milestone:** M0
**Depends on:** [003]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
There is no file that tells an agent (or a new human) what Rix is, where the plan lives, how
to build, test and verify, and which rules hold. Every session starts from zero.

## Approach
Write `AGENTS.md` at the repo root: what Rix is (a Qix-inspired arcade game in Bevy), the
release goal (D1), where to start (tracker index, ARCHITECTURE.md), tracker discipline
(copied from mud and ketania), the commands table, code standards, git rules (D4) and the
verification culture. No `CLAUDE.md` (D5): Claude Code reads `AGENTS.md` when no
`CLAUDE.md` exists.

## Verification
`AGENTS.md` exists at the root, has the sections above, and every command in its table
runs from a clean checkout.

## Notes
- Pedro asked for `AGENTS.md` as the source of truth, with a `CLAUDE.md` symlink only if
  Claude Code does not read `AGENTS.md`. It does (D5), so there is no symlink.
- Done. `AGENTS.md` at the root: what Rix is, goal (D1), start here, tracker discipline
  (from mud), hard rules (D2, D3, asset licences, no secrets, seeded RNG), code standards,
  git (D4, D8), commands, verification culture, platform notes (macOS 27 objc2).
- Commands checked in the working tree on 2026-11-02 (not in a fresh clone): `cargo build`,
  `cargo build --release` (1m 37s warm), `cargo test` (32 pass), `cargo clippy --all-targets
  --features dev_tools` (15 pre-existing warnings), `cargo fmt --check` (fails on old code,
  as the file says), `just shot`, `just shot-menu`, `cargo run --features dev` (= `just
  dev`) with `RIX_SHOT` set so it exits, `cargo build --examples` (the examples open a
  window and do not exit, so only their build is checked), and the `tracker/` commands.
