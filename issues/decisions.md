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

## Gate answers

None yet.
