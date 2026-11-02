# Rix Issues

Next ID: 077

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

None.

## Open (priority order)

*Order set 2026-11-02: the dev harness and tests first (the upgrade needs them), then the
upgrade (M1, D3) with lyon replaced first (D2), then the gates so Pedro can answer them
while M2 runs, then mobile (D1: iOS and Android are the goal; Android first, it has the
known crash).*

*"Continue" takes the top of this list. A line marked **gate** or **owner-run** waits for
Pedro: take the next one.*

- [025] M2: Device smoke on a real iPhone and Android phone; **owner-run**
- [032] M3: Data-driven enemy spawning with weights and tiers
- [033] M3: Items spawn during play; a caught item keeps its kind
- [035] M3: Dead code, lint table and clippy clean
- [036] M4: Menu UI redesign
- [037] M4: HUD redesign
- [038] M4: Game over and level transition screens and animations
- [039] M4: Visible enemy death when eaten
- [040] M4: Juice and FX pass
- [076] M4: Smooth motion on 90 and 120 Hz screens
- [041] M4: Audio system - music and SFX channels
- [042] M4: Sound effects set
- [043] M4: Music
- [044] M4: New enemy - Splitter
- [045] M4: New enemy - Shooter
- [046] M4: New items - multiplier, stop time, auto-line
- [069] M4: New item - Star (rainbow line, invulnerable)
- [047] M4: Difficulty curve and balance pass; **owner-run** sign-off
- [048] M4: Settings, high score and stats persistence
- [067] M4: Show, share and play a seed
- [075] M4: Replay without the renderer, checked on two targets in CI
- [068] M4: Leaderboards on Game Center and Play Games
- [073] M4: Pause button and pause menu
- [049] M4: First-run tutorial
- [070] M4: Progressive unlocks with a tip on first sight
- [071] M4: Wiki - enemies and items explained
- [050] M5: App icon set
- [051] M5: Store art - screenshots, feature graphic, preview video
- [052] M5: Privacy, ratings, credits and licences
- [053] M5: iOS release - signing, App Store Connect, TestFlight; accounts **owner-run**
- [054] M5: Android release - AAB, signing, Play Console, closed test; accounts **owner-run**
- [055] M5: Device QA matrix; **owner-run**
- [056] M5: Ship mobile v1.0; **owner-run**
- [057] M6: Web canvas and input
- [058] M6: Web release build and CI release workflow
- [059] M6: itch.io page and butler upload; page **owner-run**

## Milestones
- [x] M0 Foundations
- [x] M1 Engine upgrade
- [ ] M2 Mobile builds
- [ ] M3 Core rework
- [ ] M4 Content and polish
- [ ] M5 Mobile release
- [ ] M6 itch.io release

## Milestone goals

- **M0 Foundations:** an agent or a human can pick up Rix cold: `AGENTS.md`,
  `ARCHITECTURE.md`, this tracker and its board, a dev harness that proves the game runs,
  tests for the pure logic, CI, the owner gates asked.
- **M1 Engine upgrade:** Rix on Bevy 0.20.1 and avian2d 0.8, one minor version per step
  (D3), with no `bevy_prototype_lyon` (D2) and the game behaving as before.
- **M2 Mobile builds:** iOS and Android build from main with one command each, run on the
  simulator and the emulator without crashing, handle safe areas and suspend/resume, play
  the same on every screen size, and pass a device smoke.
- **M3 Core rework:** idiomatic structure (bevy_new_2d patterns, D7), one place for assets,
  clean state, a robust territory model, the known bugs fixed, clippy clean with a lint
  table.
- **M4 Content and polish:** menus, HUD, transitions, FX, audio, the v1 enemies and items
  from [012], a tuned difficulty curve, persistence, a tutorial.
- **M5 Mobile release:** icon, store art, privacy and credits, signed builds, TestFlight and
  the Play closed test, device QA, v1.0 on both stores.
- **M6 itch.io release:** the web build on itch.io, with a canvas that fits the page and a
  release workflow. No desktop downloads (D13), no Steam, no GOG (D9).
