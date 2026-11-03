# [102] Rix site on Vercel: landing page and privacy policy

**Status:** done
**Milestone:** M5
**Depends on:** [052]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
- 2026-11-03, deploy: Pedro logged in (`yonson2`, scope `yonson2s-projects`).
  `vercel project add rix` first, then `vercel link --yes --project rix --cwd site`: a
  link with `--yes` that creates the project also connects the Git repository (every
  push would deploy), and linking an existing project does not. `inspect` shows no Git
  connection. The link writes `site/.env.local` (a `VERCEL_OIDC_TOKEN`, not needed for a
  static site) and `site/.gitignore`: both deleted; the root `.gitignore` covers
  `site/.vercel/` and `site/.env*`. `just site-deploy` is now
  `npx --yes vercel deploy --prod --yes --cwd site` (the link is read from `site/`).
  Production: https://rix-tau.vercel.app (`/`, `/privacy`, the font, the CSS and the
  icon answer 200); the `*-yonson2s-projects.vercel.app` URLs redirect to a Vercel login
  (deployment protection, default).
- Domain: `vercel domains add rix.cosas.top rix` (verified, current scope). The record:
  CNAME `rix` -> `c28c6379a775714a.vercel-dns-017.com` (Pedro's other subdomains,
  mtgp, yalp and sparky, use the same kind of record). Owner-run at Porkbun.
- 2026-11-03, live: Pedro added the CNAME at Porkbun. `rix.cosas.top` resolves to
  `c28c6379a775714a.vercel-dns-017.com.`; https://rix.cosas.top/ ("Rix") and
  https://rix.cosas.top/privacy ("Rix privacy policy", contact link to the GitHub issues
  page) answer 200 with a valid certificate, also `/fonts/main.ttf`, `/style.css` and
  `/icon.png`. This also passes [052]'s privacy URL check. [053] and [054] already name
  the URL.
