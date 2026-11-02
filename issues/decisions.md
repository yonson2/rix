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
| D17 | 2026-11-02 | **The fixed playfield is tall 9:16** (Pedro, [019]). It fills a portrait phone and letterboxes in landscape (a strip in the middle). Speeds and spawn counts get a retune for the new shape. |
| D18 | 2026-11-02 | **Three more M4 features** (Pedro's ideas): a Star item (rainbow line, invulnerable, music change) [069]; progressive unlocks with a tip the first time each kind spawns [070]; a wiki of enemies and items [071]. |
| D19 | 2026-11-02 | **Territory model A** ([029], Pedro): one free-region polygon that each line cuts in two; a line ends within 1 unit of the free boundary and snaps to it. Claimed pieces stay lyon meshes with trimesh colliders; progress is the exact free area. Not the grid model (stepped edges, about 1 ms per claim, new render and collider code). Built in [074]. |
| D20 | 2026-11-03 | **Unlockable skins in M4** (Pedro's idea): skins for the line (for example a chain) and for the walls, unlocked by reaching a level or a score; look only [078]. |

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
