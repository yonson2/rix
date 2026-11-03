# [102] Rix site on Vercel: landing page and privacy policy

**Status:** active
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
  the game, store links when they exist), `/privacy` the policy.
- The neon look of the game, plain HTML and CSS, the game font.
- Deploy: a Vercel project on that folder and the domain `rix.cosas.top` (DNS at
  cosas.top). Pedro's Vercel account: **owner-run** or with his go.

## Verification
https://rix.cosas.top/ and https://rix.cosas.top/privacy answer 200 and show the landing
page and the policy.

## Notes
- From [052] (2026-11-03): this issue's Verification is also [052]'s privacy URL check.
  The policy's contact is the GitHub issues page (D35). Deploy with the Vercel CLI; Pedro
  logs in once (D35).
- 2026-11-03: the site is in `site/`: `index.html` (the logo box in CSS, the tagline, what
  Rix is, "coming soon"), `privacy.html` (the policy; `PRIVACY.md` is gone, so there is
  one copy), `style.css` (the game's colours, the game font for headings), `fonts/` (a
  copy of `assets/fonts/main.ttf` with its OFL text), `icon.png` (180 px, from
  `tools/draw-icon.py`), `vercel.json` (`cleanUrls`, so `/privacy` serves `privacy.html`).
  Checked in headless Chrome at 390 px (in an iframe: the window has a minimum width)
  and at 1280 px. `just site-deploy` runs `npx vercel deploy site --prod`.
- DNS: cosas.top is at Porkbun; `rix.cosas.top` resolves today to 65.108.240.24 (a
  wildcard record), so it needs its own CNAME to Vercel's target (owner-run at Porkbun).
