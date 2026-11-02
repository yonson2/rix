# [005] Infra: Issue board first Vercel deploy

**Status:** done
**Milestone:** M0
**Depends on:** [004]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
The board ([004]) only runs locally until it is deployed somewhere private.

## Approach
Pedro creates a Vercel project from `yonson2/rix` with Root Directory `tracker`, and sets
`BASIC_AUTH_USER` and `BASIC_AUTH_PASSWORD` as runtime env vars (never build-time, or the
password lands in the bundle). Same setup as mud's issue 129.

## Verification
The deployed URL answers 401 without credentials and shows the board with them.

## Notes
- **owner-run**: needs Pedro's Vercel account.
- Before the first deploy: review `npm audit` in `tracker/` (7 findings, 6 high, all
  build-time dependencies, [004] Notes) and update what has a safe fix.
- Dropped on 2026-11-02: Pedro runs the board locally only (D10). `tracker/middleware.ts`
  (basic auth for a hosted deploy) stays; it does nothing in `npm run dev`.
