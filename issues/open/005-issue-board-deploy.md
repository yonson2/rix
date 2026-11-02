# [005] Infra: Issue board first Vercel deploy

**Status:** open
**Milestone:** M0
**Depends on:** [004]
**Created:** 2026-11-02
**Closed:** -

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
