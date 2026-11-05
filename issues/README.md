# Rix Issues

Next ID: 126

History sits beside this index, read on demand and never at session start:
`done-log.md` lists every closed issue, newest first, and `decisions.md` holds Pedro's
decisions (the D rows and gate answers) with their dates. Record a new owner decision there.

Started 2026-11-02 from Pedro's brief (engine upgrade, Android crash, polish, architecture,
tracker, AGENTS.md, release on iOS and Android first) and from `todo.md`, whose lines now
live in issues ([003] Notes has the mapping).

## How issues come into being

Issues are units of work and decisions; milestones (below) are the build-order spine they
serve.

**Milestone intake** - mandatory when starting a milestone, before writing code:
1. Read the milestone's goal below.
2. Inventory existing issues: everything open that names this milestone or feeds it.
3. Decompose the remaining work; for every item not covered by an existing issue, file a
   new one (template, runnable Verification line, Milestone + Depends on fields). Ask
   Pedro when decomposition involves a real choice.
4. Get the milestone's gate issues answered, then start from the top of the open pile.

**Gates:** a gate issue (title starts `Gate:`) holds questions only Pedro answers. Issues
that depend on it wait for the answers, even in auto mode. The answers go into
`decisions.md`, and the gate issue closes.

**Mid-work discoveries:** the moment untracked work, a follow-up, or a dead end appears,
file it as an open issue before continuing. Never do untracked work; never leave a
discovery only in chat. Small spin-offs get filed, not absorbed.

**Don't file** what an existing issue already covers - update that issue instead.
Duplicates rot the index.

Status flow: `open/ -> active/ -> done/` (move the file, update this index in the same
commit; a close also adds the issue's line at the top of `done-log.md`). Every issue
keeps the template's Verification line runnable.

## Active

- [053] M5: iOS release - 1.0.1 (10001) uploaded; App Store Connect forms **owner-run**
- [054] M5: Android release - signed AAB (`just android-release`); Play Console and closed test **owner-run**

## Open (priority order)

*Order set 2026-11-02: the dev harness and tests first (the upgrade needs them), then the
upgrade (M1, D3) with lyon replaced first (D2), then the gates so Pedro can answer them
while M2 runs, then mobile (D1: iOS and Android are the goal; Android first, it has the
known crash).*
*2026-11-05 (D39): after M5 come M7 to M9 (1.1 to 1.3), then M6 ([109] Q7).*

*"Continue" takes the top of this list. A line marked **gate** or **owner-run** waits for
Pedro: take the next one.*

- [055] M5: Device QA matrix; **owner-run**
- [056] M5: Ship mobile v1.0; **owner-run**
- [068] M7: Leaderboards on Game Center and Play Games; after 1.0 (D37), needs the store IDs (D25)
- [110] M7: Score bonuses: overclaim and multi-eat; before the boards go live (D39)
- [111] M7: Medals and the RECORDS screen
- [112] M7: Medals as Game Center and Play Games achievements; needs [068], achievements **owner-run**
- [115] M8: Claimed-area fills
- [116] M8: Kill effects
- [117] M8: Two more skins: a wall and the GOLD line
- [114] M8: Unlocks for games played (5 to 1000 games)
- [113] M8: Records: lifetime totals and the next unlock
- [118] M8: Kill counts in the wiki
- [119] M8: Secret looks
- [120] M9: Challenge rules in the seed code
- [121] M9: Challenge rule effects
- [122] M9: Countdown challenges
- [123] M9: The CHALLENGES screen
- [124] M9: Daily seed (a daily challenge)
- [125] M9: Challenge and daily leaderboards (one per challenge); boards **owner-run**
- [057] M6: Web canvas and input
- [058] M6: Web release build and CI release workflow
- [059] M6: itch.io page and butler upload; page **owner-run**

## Milestones
- [x] M0 Foundations
- [x] M1 Engine upgrade
- [x] M2 Mobile builds
- [x] M3 Core rework
- [x] M4 Content and polish
- [ ] M5 Mobile release
- [ ] M6 itch.io release
- [ ] M7 Scores and medals
- [ ] M8 Records and unlocks
- [ ] M9 Challenges

## Milestone goals

- **M0 Foundations:** an agent or a human can pick up Rix cold: `AGENTS.md`,
  `ARCHITECTURE.md`, this tracker and its board, a dev harness that proves the game runs,
  tests for the pure logic, CI, the owner gates asked.
- **M1 Engine upgrade:** Rix on Bevy 0.20.1 and avian2d 0.8, one minor version per step
  (D3), with no `bevy_prototype_lyon` (D2) and the game behaving as before.
- **M2 Mobile builds:** iOS and Android build from main with one command each, run on the
  simulator and the emulator without crashing, handle safe areas and suspend/resume, use
  the whole screen (D21), and pass a device smoke.
- **M3 Core rework:** idiomatic structure (bevy_new_2d patterns, D7), one place for assets,
  clean state, a robust territory model, the known bugs fixed, clippy clean with a lint
  table.
- **M4 Content and polish:** menus, HUD, transitions, FX, audio, the v1 enemies and items
  from [012], a tuned difficulty curve, persistence, a tutorial.
- **M5 Mobile release:** icon, store art, privacy and credits, signed builds, TestFlight and
  the Play closed test, device QA, v1.0 on both stores.
- **M6 itch.io release:** the web build on itch.io, with a canvas that fits the page and a
  release workflow. No desktop downloads (D13), no Steam, no GOG (D9).
- **M7 Scores and medals (1.1, D39):** a score that rewards skill (overclaim, multi-eat),
  the leaderboards on Game Center and Play Games, and medals that are also platform
  achievements.
- **M8 Records and unlocks (1.2, D39):** a RECORDS screen with the lifetime totals, new
  looks (claimed-area fills, kill effects, a wall, the GOLD line) that unlock at 5 to 1000
  games, kill counts in the wiki, and secret looks.
- **M9 Challenges (1.3, D39):** challenges as special seeds whose codes carry the rules, a
  CHALLENGES screen with a curated list, local bests and challenges that open with games
  played, countdown challenges, a daily challenge, and a board per challenge.
