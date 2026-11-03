# [038] Game over and level transition screens and animations

**Status:** done
**Milestone:** M4
**Depends on:** [026], [030]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
Death jumps straight to a text screen ("YOU DIED!", score, level, kills, "Tap to play
again."); a level clear only fades the claimed shapes away. todo.md asks for "animations
for next level and game over" and a better game over screen.

## Approach
Death: freeze, flash the line that was hit, then the game-over screen with score, best,
level, kills, seed (if [012] Q4 says so), Retry and Menu buttons. Level clear: a short
celebration (claimed area sweeps, "LEVEL N" banner) before the next level.

## Verification
`cargo test` (the death sequence) passes; `just shot-flow` retries through the RETRY
button; flow shots at 390x844 show the "LEVEL N" banner and the game over screen; on the
Android emulator a death shows the game over screen, RETRY starts a run and MENU shows
the title, whose PLAY starts a run. Phone taps: [025].

## Notes
- todo.md: "animations for next level and gave over"; "game over screen".
- [012] Q4 (2026-11-02): the game over screen shows the seed, with a share action ([067]).
- [019] (2026-11-02): the game over screen is still Bevy UI in window pixels: it does not scale with the field. The HUD moved to world `Text2d` ([019]); do the same or use `UiScale` here.
- Done 2026-11-03:
  - Death: `handle_die` enters the new `IsPaused::Dying` (inside `Phase::Playing`, so the
    frozen level stays on screen). `death.rs` stops virtual and physics time (the pause's
    `stop_time`), flashes the line segments `NEON_BURGUNDY` / `NEON_WHITE` every 0.1 s
    on real time, and after 1 s goes to `Screen::GameOver`; leaving `Dying` starts time
    again.
  - Game over: world `Text2d` and `widget::button`s (bloom and CRT, like the title):
    "YOU DIED!", score, "LEVEL N   KILLS K", "SEED S", RETRY (new random run) and MENU
    (title with a fresh run, so PLAY starts it; `RIX_SEED` still sets only the first run).
    The tap anywhere is gone.
  - Level clear: a "LEVEL N" banner in `NEON_CYAN` pops in (`EaseFunction::BackOut`,
    0.25 s), holds, and fades in the last 0.3 s of 1.2 s; the next level waits for it and
    the shape fade.
  - Flow scene: die at 3.5 s, tap RETRY (`game_over::RETRY_Y` through the camera) at 5 s,
    shot at 6 s.
- Moved out: the best score on the game over screen needs saved data ([048]); the share
  action for the seed is [067].
- Not done: "claimed area sweeps" on a level clear. The shapes keep their 0.3 s fade with
  a drift; a bigger celebration belongs to the FX pass ([040]).
- Android emulator (2026-11-03): a line drawn through the ball ends in the game over
  screen (score, level, kills, seed, RETRY, MENU); MENU shows the title and PLAY starts a
  run at level 1; RETRY starts a new run. Flow shots (390x844): the banner at 2.0 s, the
  game over screen at 4.9 s. Default `just shot` md5 unchanged
  (427bcf4303bb9b8cc2ad97faddeeaef0); `just replay` passes.
