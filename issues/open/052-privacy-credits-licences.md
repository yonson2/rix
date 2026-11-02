# [052] Privacy, ratings, credits and licences

**Status:** open
**Milestone:** M5
**Depends on:** [010], [011]
**Created:** 2026-11-02
**Closed:** -

## Problem
Stores need a privacy policy URL and an age-rating questionnaire. The game must credit
what it uses (CC-BY music if kept, the font licence, Bevy and crate licences) and there is
no README. todo.md: "add README, cite inspirations".

## Approach
If Rix collects nothing (no analytics, no ads, local save only), a one-page privacy policy
saying so, hosted where Pedro chooses. A credits screen generated from a tracked
`CREDITS.md` and a licence report (`cargo about` or similar). A README with what Rix is,
how to build it, and its inspirations (Qix, Xonix, ...). Ratings questionnaires filled in
the store consoles (owner-run).

## Verification
The privacy URL loads; the credits screen lists every third-party asset and the licence
report; README builds from a clean checkout as written.

## Notes
- todo.md: "add README, cite inspirations".
- [010] answers (2026-11-02): free, no ads, no in-app purchases, English only. The privacy
  policy can say "collects nothing", except what Game Center and Play Games handle for the
  leaderboards ([068]).
- [011] answers (2026-11-02): the font `assets/fonts/main.ttf` has the Open Font License
  and ships; the OFL text and its copyright line go into the credits. Music and SFX are
  our own ([042], [043]). `assets/items/bomb_basic.png` has no known source: replace it
  (procedural or PixelLab, [046]) before release.
