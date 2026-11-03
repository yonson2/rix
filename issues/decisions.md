# Owner decisions

Pedro's decisions, with their dates. History: read it on demand, never at session start.
Where two entries conflict, the later date wins. The live state is in the issues and
`ARCHITECTURE.md`. Record a new owner decision here, with its date: a new `D<n>` row for a
product or architecture decision, a dated line under "Gate answers" for a gate.

## Decisions log

| # | Date | Decision |
|---|---|---|
| D1 | 2026-11-02 | **Main objective: release on iOS and Android.** Steam, GOG and itch.io come after, from the same Bevy codebase. |
| D2 | 2026-11-02 | **Replace `bevy_prototype_lyon` before the engine upgrade** with a small in-repo module that tessellates with `lyon_tessellation` into `Mesh2d`. Reasons: lyon has no release for Bevy 0.20 (its master targets 0.19, checked 2026-11-02), main tracks its git `master` so `cargo update` breaks the build, and the game reads territory geometry back from the render `Shape`. Built in [013]. |
| D3 | 2026-11-02 | **Upgrade Bevy one minor version at a time**, 0.15.3 → 0.16 → 0.17 → 0.18 → 0.19 → 0.20, following each official migration guide, one issue and one commit per step ([014]-[018]). avian2d moves with each step. |
| D4 | 2026-11-02 | **Git workflow:** work on a branch, one commit per issue, no `Co-Authored-By` or other AI attribution lines in commit messages. For the 2026-11-02 session the agent merges its branch into `main` when done. |
| D5 | 2026-11-02 | **`AGENTS.md` is the agent instructions file, with no `CLAUDE.md`.** Claude Code reads `AGENTS.md` when no `CLAUDE.md` exists in the working directory or above (https://code.claude.com/docs/en/memory#agents-md, since v2.1.277; v2.1.316 installed, no parent `CLAUDE.md`). A `CLAUDE.md` file would stop it from reading `AGENTS.md`. |
| D6 | 2026-11-02 | **The local Bevy fork (`../../git/bevy`, used on `feat/ios` for deferred screen edges) is not needed anymore:** Pedro's change was merged upstream. Use crates.io Bevy for iOS ([020]). |
| D7 | 2026-11-02 | **Study TheBevyFlock/bevy_new_2d and adopt the patterns that fit** (Pedro). The findings are in [026]. |
| D8 | 2026-11-02 | **The untracked files stay untracked and untouched** (`AI.md`, `splitter.md`, `credits.txt`, `assets/songs/`, `.DS_Store`): Pedro does not need them. The Splitter idea lives on as a candidate in [044]; the songs and credits as a candidate in [043]. |
| D9 | 2026-11-02 | **Release on iOS, Android and itch.io (web build) only. No Steam, no GOG** ([010] Q7). This replaces the "Steam, GOG" part of D1. [060] and [061] are dropped. |
| D10 | 2026-11-02 | **The issue board stays local** (`npm run dev` in `tracker/`). No Vercel deploy: [005] is dropped. |
| D11 | 2026-11-02 | **Delete each old branch once its work is in `main`** ([009]): `feat/touch` and `feat/splitter` now; `feat/item_outline_shader` after [040] ports the outline shader; `feat/ios` after [020]; `feat/android` after [021]. |
| D12 | 2026-11-02 | **Bump the `Cargo.toml` version as the work advances; 1.0.0 is the mobile store release** (Pedro). Rule: the minor version goes up when a milestone completes (M0 + M1 → 0.3.0, M2 → 0.4.0, M3 → 0.5.0, M4 → 0.6.0, M5 → 1.0.0, M6 → 1.1.0); the patch version goes up on each merge to `main` that changes the game between milestones. `Cargo.lock` changes with it (CI builds `--locked`). |
| D13 | 2026-11-02 | **itch.io gets the web build only.** No desktop downloads (no macOS signing or notarization, no Windows or Linux packages). The native desktop build stays a dev tool. |
| D14 | 2026-11-02 | **Git housekeeping:** the agent deletes its own work branch after it merges it into `main`. Each version bump (D12) gets an annotated tag `vX.Y.Z` on the commit that bumps it; `v0.3.0` is the first. |
| D15 | 2026-11-02 | **The untracked leftovers are resolved (replaces D8)** (Pedro asked). `splitter.md` is in git as `docs/design/splitter.md` ([044]). `AI.md` (general Bevy 0.15 notes), `credits.txt` and `assets/songs/` (incompetech mp3s, not used since [011] Q3) went to the macOS Trash, `~/.Trash/rix-leftovers-2026-11-02`. |
| D16 | 2026-11-02 | **Keys for generated assets live in `.env` at the repo root (gitignored):** `PIXELLAB_TOKEN`, `ELEVENLABS_TOKEN`, `EVOLINK_TOKEN`. `OPENROUTER_API_KEY` comes from Pedro's shell (`~/.zshrc`). Never print or commit them. |
| D17 | 2026-11-02 | **The fixed playfield is tall 9:16** (Pedro, [019]). It fills a portrait phone and letterboxes in landscape (a strip in the middle). Speeds and spawn counts get a retune for the new shape. Replaced by D21 (2026-11-03). |
| D18 | 2026-11-02 | **Three more M4 features** (Pedro's ideas): a Star item (rainbow line, invulnerable, music change) [069]; progressive unlocks with a tip the first time each kind spawns [070]; a wiki of enemies and items [071]. |
| D19 | 2026-11-02 | **Territory model A** ([029], Pedro): one free-region polygon that each line cuts in two; a line ends within 1 unit of the free boundary and snaps to it. Claimed pieces stay lyon meshes with trimesh colliders; progress is the exact free area. Not the grid model (stepped edges, about 1 ms per claim, new render and collider code). Built in [074]. |
| D20 | 2026-11-03 | **Unlockable skins in M4** (Pedro's idea): skins for the line (for example a chain) and for the walls, unlocked by reaching a level or a score; look only [078]. |
| D21 | 2026-11-03 | **The field takes the screen shape; phones are portrait only** (Pedro, [080]; replaces the fixed 9:16 of D17 and the landscape answer of [010] Q4). The field is 450 world units wide and its height follows the screen at the start of a run, so a phone uses its whole screen. A seed keeps its RNG draws; positions follow the field, so a seed plays a little differently on another screen shape. Pedro: "adapting to use all the screen on mobile seems like a much bigger win for playability than what we have now". |
| D22 | 2026-11-03 | **Music comes from Suno (v6, through EvoLink as in Ketania); sound effects stay on ElevenLabs** (Pedro, [043]). Pedro checked the licence himself: "suno does allow comercial usage through their Page. Use suno and note down that I've checked that the license is fine. Suno > elevenlabs for music (sfx is fine for elevenlabs)". Eleven Music is not used. |
| D23 | 2026-11-03 | **[070] is tips only, no locks** (Pedro). The level gates inside a run stay (Splitter from level 2, Shooter from 3, items from 2 to 4); the first time a player meets a kind, a short tip explains it, once (saved). Nothing is locked, so a shared seed gives the same game to everyone. This narrows the "progressive unlocks" part of D18. |
| D24 | 2026-11-03 | **Skins v1 is a small set** (Pedro, [078]): 2 line skins (chain, dashed laser) and 2 wall skins (circuit board, neon brick), unlocked at best level 5, 10, 15 and 20, chosen on a skins screen from the title. The Star's rainbow line ([069]) wins over a line skin while it lasts. |
| D25 | 2026-11-03 | **Leaderboards wait for Pedro's store accounts** (Pedro, [068]): the code starts when the App Store Connect leaderboard ID and the Play Games app ID exist ([053], [054]). Music: Pedro picks the style and the tracks by ear on the listening page ([043]); the agent does not pick for him. |
| D26 | 2026-11-03 | **The soundtrack is a reward; music stays off until the player turns it on** (Pedro, [089]). A new player hears only the sound effects. A best level unlocks the soundtrack; then the MUSIC setting opens at 0 %, and the player turns it on in the settings. Pedro: "at first we should only have sfx and then, at some point, also unlock the soundtrack (the thing I want to make with this is more rewards and a way to have sounds disabled by default unless the user enables them conciously)". The Star plays two tracks ([088]): Pedro's pick `chiptune-star-2a` and `chiptune-star-1b` ("I also like 1B alongside 2A so let's have that song too"). |
| D27 | 2026-11-03 | **Feedback on v0.4.20** (Pedro, [090] to [095]): more space between the title buttons; a locked MUSIC button is gray with a diagonal line across it, as the bar of a no sign, and no "LEVEL 3" text; the whole first run (the run with the first-run tutorial) unlocks nothing, so the best level counts from the second run; the laser line becomes a "hot beam" (white-hot core, red glow, pulses that run to the finger, a flicker, a bright tip); a share carries the score and a picture of the field at the death. |
| D28 | 2026-11-03 | **The Star's line kills enemies** (Pedro, [096]): "while its active, if an enemy crosses the line it should get destroyed instead of the line broken". An enemy that touches the line during the Star dies as a claim kills it (points, kill count, explosion). Pedro on [095]: "The screenshot thing works." |
| D29 | 2026-11-03 | **[025] and [047] close; M5 starts** (Pedro). [025]: "I've played on android, let's assume ios works too for now": the iPhone run is not done, Pedro accepts the risk. [047]: "it feels ok for now": the curve and the soundtrack level 3 stay. [068] keeps waiting for the store accounts (D25). M5: [052] "yes, do it", [050] "do it", [051] "ok". |
| D30 | 2026-11-03 | **The level change does not stop the game** (Pedro, [097]): no "LEVEL N" banner and no wait for it; a touch on the wall during the change starts the line. Pedro: "this should be a fast-paced game, waiting for that screen to finish kills the game". |
| D31 | 2026-11-03 | **The app icon is the title's logo box** (Pedro, [050]): candidate A of four on the picking page (the RIX box in the game font, purple fill, glowing edge, on dark), picked with no note. Drawn by `tools/draw-icon.py`. |
| D32 | 2026-11-03 | **iPhone only; store shots get captions; no preview video** (Pedro, on [051]): "let's go iphone only" (the Xcode target drops iPad, so the App Store needs no iPad shots), "Add captions" on the store screenshots, preview video "No" ([100] dropped). M6 (web) does not start while M5 waits on the store accounts: "No". The credits name is "Pedro". |
| D33 | 2026-11-03 | **The privacy policy lives at rix.cosas.top/privacy** (Pedro, [052]): a small static site on Vercel, `/` the landing page and `/privacy` the privacy page. That URL goes into both stores. |
| D34 | 2026-11-03 | **Shaders: written from scratch; unused ones go** (Pedro, [099]): "Delete unused shaders, otherwise they have been programmed from scratch looking at other references". The flame shader (example only) is deleted. [086]: the CRT and the first kill or bomb look "good" on Pedro's Android phone. |
| D35 | 2026-11-03 | **Rix is free software under the GPL 3.0** (Pedro, [052]). Pedro first asked for "Open source but no one can use it commercially, must be free and open source"; told that an OSI open-source licence cannot forbid commercial use, he picked GPL-3.0 over PolyForm Noncommercial and CC BY-NC-SA. `LICENSE`, `license = "GPL-3.0-only"`; the code and Rix's own art. Privacy contact: the GitHub issues page. The site goes up with the Vercel CLI, Pedro logs in ([102]). |
| D36 | 2026-11-03 | **Store links on the site when the store pages exist** (Pedro, [102]): the landing page gets the App Store and Google Play links then, with a `just site-deploy`. [104] starts now (Pedro: "yes, now"). |

## Gate answers

### [010] Product decisions for v1 (2026-11-02)
1. Name: "Rix" is final, also if a store already has a game with that name.
2. Bundle / application id: `top.cosas.rix`.
3. Price: completely free. No ads, no in-app purchases.
4. Orientation: portrait and landscape.
5. Playfield: one fixed playfield on every device. A seed gives the same game on any
   device ([019]).
6. OS floors: Pedro does not mind. Newer floors are fine when they make the work simpler:
   take the floor that is least work with Bevy 0.20 ([020], [021]).
7. Web build on itch.io: yes. Steam and GOG are dropped (D9).
8. Languages: English only.

### [011] Art and audio direction (2026-11-02)
1. Keep the neon-vector look with bloom and the CRT pass. References: Geometry Wars and
   Hyper Light Drifter.
2. Font `assets/fonts/main.ttf`: it has the Open Font License, so it ships (with its
   licence text in the credits, [052]).
3. Music: generate our own, as in Ketania. The incompetech tracks are not used.
4. SFX: generate our own too.
5. Images: procedural shapes where they fit, a PixelLab image where they do not.
6. App icon: a new design (flux, PixelLab or similar). The `feat/ios` icon is not used.

### [012] v1 content scope (2026-11-02)
1. New enemies for v1: Splitter and Shooter (for now).
2. New items for v1: all three (point multiplier, stop time, auto-line).
3. Structure: endless levels; one death ends the run (as today).
4. Seeds: show and share them in v1. Each run gets a new seed; no daily seed.
5. Leaderboards in v1: one platform-agnostic layer that submits to Game Center (iOS) and
   Play Games (Android).
