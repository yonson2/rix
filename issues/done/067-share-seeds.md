# [067] Show, share and play a seed

**Status:** done
**Milestone:** M4
**Depends on:** [019], [026], [036], [038]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
[012] Q4: v1 shows and shares seeds. Each run gets a new seed (no daily seed). Today the
seed lives only in `LevelInfo.seed`; a player cannot see it, send it, or play a seed that
someone sent.

## Approach
Show the run seed on the game over screen ([038]) and in the pause menu ([034]), as a short
code (for example base32 of the `u64`). Share it with the OS share sheet on iOS and
Android, and copy it to the clipboard on web and desktop. A "Play seed" entry in the menu
([036]) takes a typed or pasted code and starts a run with it. The same seed must give the
same game on every device: that needs the fixed playfield ([019]) and a fixed system order
([026]).

## Verification
A unit test round-trips seed <-> code. `RIX_SEED=<n>` and "Play seed" with the code of
`<n>` give the same `RIX_STATE` hash (a shot run patched to type the code). The game over
screen shows the code.

## Notes
- From [012] Q4 (Pedro, 2026-11-02).
- [072] (2026-11-03): a seed now gives the same state at any frame rate (`just replay`); a
  check across targets is [075].

## Result (2026-11-03)
- `src/game/seed.rs`: seed codes in Crockford base32, in groups of four (`K3M9-ZQ4T`).
  - `code` writes at least eight characters; `parse` ignores dashes, spaces and case, and
    reads I and L as 1 and O as 0; `find` takes the code out of a pasted message.
  - A new run's seed now has 40 bits (`random_seed`), so its code has eight characters.
    `RIX_SEED` still takes any `u64` (seed 1 is `0000-0001`).
- The code shows on the game over screen (in place of the number) and in the pause overlay.
- Game over: SHARE sends "Rix seed CODE". On phones it opens the share sheet; on desktop
  and the web the button says COPY, copies, then says COPIED.
- Title: PLAY SEED opens `Screen::Seed`: a keypad of the 32 code characters, DEL, PASTE,
  PLAY and BACK; a keyboard types too. PLAY starts a run with the code's seed.
- `src/share.rs`: share and paste per platform.
  - Android: `MainActivity.share` (an `ACTION_SEND` chooser) and `pasteText` over JNI.
  - iOS: `UIActivityViewController` (with a popover source for the iPad) and
    `UIPasteboard`.
  - Desktop: Bevy's clipboard with the `system_clipboard` feature. Web: Bevy's async
    clipboard read (not checked here; the web build is [057]).
- Dead end: the `system_clipboard` feature for all targets breaks the iOS build. On iOS,
  `arboard` takes its Linux path (`wl-clipboard-rs`, `x11rb`), which turns on `rustix`
  0.38's `net` feature, and `libc` 0.2.191 has `AF_SYS_CONTROL` only for macOS. The
  feature is now in a desktop-only dependency table.

## Evidence
- `cargo test` (90 pass, new: seed round trip, loose parse, find, keypad typing), clippy
  (desktop with `dev_tools`, the iOS simulator target, Android through cargo-ndk), fmt.
- `just replay`: the five hashes are unchanged. `just shot`: md5 `839615a6…` unchanged.
- A shot run patched to type `0000-0001` on the seed screen and press Enter (`RIX_SEED`
  unset) logs `playing seed 0000-0001 (1)` and `RIX_STATE` `7f4efff7fa02493e`, the hash
  of `RIX_SEED=1`. Its picture, timed from gameplay start like a `RIX_SEED=1` run, differs
  only by a one-pixel shift of the falling square: the transform interpolation has
  another fixed-clock phase after three menu frames. The hash covers each enemy's exact
  position and velocity, the free region, the points, the kills and the RNG state.
- Desktop clipboard: a patched run triggered `Share` and `Paste`. `pbpaste` gave
  "Rix seed k3m9-zq4t" and the seed screen showed `K3M9-ZQ4T`.
- Android emulator (debug APK): the game over showed `SEED XFKC-YG0D` and SHARE; SHARE
  opened the chooser with "Rix seed XFKC-YG0D"; its copy button, then PLAY SEED and PASTE,
  showed `XFKC-YG0D`; PLAY logged `playing seed XFKC-YG0D (1013189656589)`; Back paused
  with the seed line. An empty clipboard logs "paste failed: clipboard contents were
  unavailable" (`pasteText` returns null), no JNI error. The headless emulator does not
  sync the Mac clipboard.
- iOS simulator: builds (`just ios-sim`), starts and shows PLAY SEED. The share sheet
  and paste on iOS are not run: the simulator cannot be tapped from the command line. The
  check is in [025].
- Shots: `target/shots/s067-title.png`, `s067-seed.png`, `s067-gameover.png`,
  `s067-paste.png`, `s067-android-{gameover,share,pasted,pause}.png`.
