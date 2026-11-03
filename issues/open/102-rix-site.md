# [102] Rix site on Vercel: landing page and privacy policy

**Status:** open
**Milestone:** M5
**Depends on:** [052]
**Created:** 2026-11-03
**Closed:** -

## Problem
Both stores need a privacy policy URL. Pedro (D33): rix.cosas.top/privacy, from "a small
static site that will be deployed to vercel (/ is the landing page, /privacy is the
privacy page)".

## Approach
- A static site in the repo (no build step): `/` a landing page (the logo, one line on
  the game, store links when they exist), `/privacy` the policy from `PRIVACY.md`.
- The neon look of the game, plain HTML and CSS, the game font.
- Deploy: a Vercel project on that folder and the domain `rix.cosas.top` (DNS at
  cosas.top). Pedro's Vercel account: **owner-run** or with his go.

## Verification
https://rix.cosas.top/ and https://rix.cosas.top/privacy answer 200 and show the landing
page and the policy; the policy matches `PRIVACY.md`.

## Notes
