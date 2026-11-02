# [001] AGENTS.md - agent working agreements and the game in one page

**Status:** open
**Milestone:** M0
**Depends on:** [003]
**Created:** 2026-11-02
**Closed:** -

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
