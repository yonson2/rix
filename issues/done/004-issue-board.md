# [004] Issue board - port mud's Astro board to `tracker/`

**Status:** done
**Milestone:** M0
**Depends on:** [003]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
The markdown tracker is hard to scan: no board view of open, active and done issues, no
milestone progress. mud and ketania both render theirs as a read-only Astro site.

## Approach
Port `~/projects/mud/tracker/` (itself ported from ketania) into `tracker/`: same loader
(reads `issues/{open,active,done}/*.md` and the README's milestones and open order; a file
that does not parse fails the build), same basic-auth edge middleware, same noindex
headers. Drop mud-only parts (M9.5 parsing quirks, the music-poster components). Restyle to
Rix: black background, the game's neon palette (`src/colors.rs`), arcade/CRT feel. HTML
and CSS only, no client JS islands.

## Verification
In `tracker/`: `npm ci && npm test && npm run check && npm run build` pass, and the built
board lists every issue in `issues/` with the right status and milestone.

## Notes
- Deploying it is [005] (owner-run: Vercel project + basic-auth env vars).
- 2026-11-02 done: mud's board ported to `tracker/`. Same strict loader, basic-auth edge
  middleware (fails closed, realm "Rix issue board"), noindex headers. Dropped music parts
  (vinyl, tape chart, player, Phosphor icons, web fonts) and mud quirks (M9.5, off-spine
  labels). An issue whose milestone is not in the README goes to a "No milestone" group.
  Black + `src/colors.rs` neon palette, system mono, CSS scanlines; goals shown per
  milestone (own parser for `## Milestone goals`). No scripts and no external requests in
  `dist/`.
- Check on the branch with [013] and [062]: `npm ci && npm test && npm run check && npm
  run build` pass (39 tests, 0 check errors, 65 pages); 62 issue files = 62 issue pages;
  board columns: Shipped 003, 006, 007, 013; Backlog 58.
- `npm audit`: 7 findings (6 high) in build-time dependencies (vite, sharp, undici, zod,
  devalue, source-map-js, smol-toml), same versions as mud's lock. Not fixed: [005] Notes.
