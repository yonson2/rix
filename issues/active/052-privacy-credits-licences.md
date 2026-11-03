# [052] Privacy, ratings, credits and licences

**Status:** active
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
- From [036] (2026-11-03): the credits screen gets a CREDITS button on the title
  (`theme::widget::button`, `src/screens/title.rs`).
- 2026-11-03 (D29, Pedro: "yes, do it"): first pass.
  - Credits screen `src/screens/credits.rs` (`Screen::Credits`, `RIX_SHOT_SCENE=credits`);
    CREDITS on the title's last row (whole row on phones and the web, half beside QUIT on
    desktop; `widget::label_size` shrinks a label to its button's width).
  - `CREDITS.md`: the asset table (font, sound effects, music, shaders) and the code.
  - The font is "balatro" by localthunk, made with FontStruct
    (fontstruct.com/fontstructions/show/2326420/balatro), name table: Open Font License.
    `assets/fonts/OFL.txt` (official OFL 1.1 text, openfontlicense.org) ships beside it.
  - Crate licences: `just licences` (`cargo-about`, `tools/about/`) writes
    `assets/licenses/third-party.md` (241 KB: 465 crates, MIT, Unicode-3.0, Apache-2.0,
    Zlib, MIT-0, BSD-3-Clause, ISC) for the Android, iOS and wasm targets. `rix` got
    `publish = false`, or cargo-about reads the font's OFL file as Rix's licence.
  - `README.md` (what Rix is, inspirations, build) and `PRIVACY.md` (collects nothing;
    no network, no permissions: the manifest has none and no code opens a connection).
  - Discoveries: [098] (bomb image, unknown source) and [099] (shader origins; the flame
    shader matches Balatro's).
  - Open: Pedro's contact for `PRIVACY.md` (placeholder `CONTACT`), the name to credit
    (the screen says "PEDRO"), where the privacy policy lives (proposal: the GitHub page
    of `PRIVACY.md`, the repo is public), and the ratings questionnaires (owner-run, in
    the store consoles with [053] and [054]).
- Verification so far (2026-11-03, v0.4.25): the privacy URL
  https://github.com/yonson2/rix/blob/main/PRIVACY.md answers 200. A clean clone of main
  runs the README's `cargo test` (131 pass) and `just shot` (md5
  e170997305a806320c6f9f967cf3cf5b, as in the main checkout); `just dev` was not run (it
  opens a window). The CREDITS screen shows on the Android emulator, its button the whole
  last row of the title.
