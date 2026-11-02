# [004] Issue board - port mud's Astro board to `tracker/`

**Status:** open
**Milestone:** M0
**Depends on:** [003]
**Created:** 2026-11-02
**Closed:** -

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
