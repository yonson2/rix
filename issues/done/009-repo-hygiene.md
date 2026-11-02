# [009] Repo hygiene - stale branches, tracked .DS_Store, build outputs

**Status:** done
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
- Branches: `feat/touch` (4 commits, 74 behind main; its input abstraction was superseded
  by `src/rix/user_input.rs`), `feat/splitter` (0 commits ahead), `feat/item_outline_shader`
  (outline shader + example, 3 behind; also commits the mp3s), `feat/ios` and
  `feat/android` (ported in [020], [021]).
- `assets/fonts/.DS_Store` is tracked.
- `feat/android` committed a 73 MB debug `librix.so`, `.gradle/`, `.idea/` and `.cxx/`.
  `.gitignore` does not stop that from happening again.
- `examples/*.rs` still exist on main although commit 3fba2d6 says "remove examples"; check
  whether they build.

## Approach
Extend `.gitignore` (`.DS_Store`, `/build`, `*.so` under jniLibs, `.gradle/`, `.idea/`,
`.cxx/`, `local.properties`, Xcode `xcuserdata/`). Untrack `assets/fonts/.DS_Store`.
Ask Pedro before deleting any branch (list above). Decide examples: keep if they build and
serve shader work, else delete.

## Verification
`git ls-files | grep -E 'DS_Store|\.so$|\.gradle|\.cxx'` is empty; `cargo build --examples`
passes or the examples are gone; Pedro answered on each branch.

## Notes
- Deleting branches needs Pedro's OK.
- 2026-11-02, local part done: `.gitignore` now covers `.DS_Store`, `**/jniLibs/**/*.so`,
  `.gradle/`, `.idea/`, `.cxx/`, `local.properties` and `xcuserdata/` (`build` and
  `rix.xcodeproj` were there already). `assets/fonts/.DS_Store` is untracked (the file
  stays on disk). The `git ls-files` check prints nothing.
- Examples: kept. `cargo build --examples` passes on Bevy 0.20.1, and they are the quick
  loop for shader work (AGENTS.md lists them).
- Waits on Pedro: one answer per branch (`feat/touch`, `feat/splitter`,
  `feat/item_outline_shader`, `feat/ios`, `feat/android`): delete, keep, or delete after
  [020] / [021] port it.
- Pedro (2026-11-02, D11): delete each branch once its work is in `main`.
  - `feat/touch` deleted: its mouse/touch input layer is covered by `user_input.rs` (one
    `InputState` for both). Tip was `1168133ee3e76961083152e127f720a81e3e4ecc`.
  - `feat/splitter` deleted: no commits of its own (0 ahead of `main`). Tip was
    `3fdeda9bb8be`, which `main` contains.
  - Restore either one with `git branch <name> <sha>` until git garbage-collects it.
  - `feat/item_outline_shader`, `feat/ios`, `feat/android`: deleted when [040], [020] and
    [021] land (each issue's Notes say so).
