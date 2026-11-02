# [003] Issue tracker - index, template, decisions, done-log, backlog filed

**Status:** done
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
The only plan is `todo.md`, a flat list of ideas. There is no record of decisions, no
order of work and no definition of done. Pedro wants a tracker like the ones in
`~/projects/mud` and `~/projects/ketania`.

## Approach
Copy mud's markdown format exactly (so its board, [004], parses it unchanged):
`issues/README.md` (index: rules, active, open pile in priority order, milestones),
`TEMPLATE.md`, `decisions.md`, `done-log.md`, one file per issue in `open/`, `active/`,
`done/`. Decompose everything Pedro listed on 2026-11-02 into milestones M0-M6 and file
one issue per unit of work, gates for owner choices. Migrate every `todo.md` line into an
issue and remove `todo.md`, so there is one plan.

## Verification
Every `todo.md` line maps to an issue (Notes of each issue say "todo.md: ..."); every
issue file has the template fields and a Verification line; `issues/README.md` lists every
open and active issue exactly once.

## Notes
- todo.md mapping: more enemies (Splitter → [044], Shooter → [045]); spawn weights and
  tiers → [032]; visible death when eaten → [039]; items (multipliers, stop time, auto-line)
  → [046]; animations for next level and game over → [038]; HUD → [037]; README and cite
  inspirations → [052]; STATS resource → [048]; square teleport loop → [032]; redo item
  spawn → [033]; item outline → [040]; menu UI → [036]; game over screen → [038].
- 2026-11-02 done: mud's format copied as is (same fields, README sections and milestone
  line shape, so [004] can reuse mud's parser). 61 issues filed across M0-M6, 3 gates
  ([010]-[012]). `todo.md` removed; every line is in an issue (mapping above).
  Check: a script over `issues/` found every open issue listed once in the index, every
  file with the template fields, sections and matching status, and no reference to a
  missing issue.
