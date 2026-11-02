# [009] Repo hygiene - stale branches, tracked .DS_Store, build outputs

**Status:** open
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

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
