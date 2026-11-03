# [046] New items - point multiplier, stop time, auto-line

**Status:** done
**Milestone:** M4
**Depends on:** [033], [012]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
One item (Basic Bomb). todo.md: "items: point multipliers, stop time, maybe something that
splits the line and it automatically tries to reach the wall."

## Approach
Each item as its own module on the [033] spawner, with a distinct icon ([011]) and
activation effect. Scope from [012] Q2.

## Verification
Each chosen item has a test or a scripted run that shows its effect.

## Notes
- todo.md line above.
- [011] Q5 (2026-11-02): item icons are procedural shapes where they fit, PixelLab images
  where they do not.
- [012] Q2 (2026-11-02): all three items are in v1: point multiplier, stop time, auto-line.

## Result (2026-11-03)
- Three items, each in its own module under `src/game/items/`, on the [033] spawner:
  - `multiplier` (gem token): kill and claim points count twice for 10 s.
  - `time_stop` (hourglass token): enemies and bullets freeze for 4 s (zero velocity,
    locked axes). They stay deadly, and a claim still eats them. Arrow chase, splitter
    wander and shooter fire wait on `time_flows`. A frozen square shows in full: a square
    frozen while faded out would kill unseen.
  - `auto_line` (arrow token): the line in progress shoots on along its last 40 units to
    the free boundary and claims. A shot that would cross the line does not fire.
- Icons are procedural `Shape`s ([011] Q5) on a dark token with a white frame. The
  multiplier is a gem, not a star: the star is [069]'s item.
- `pick_item`: an even pick among the items of the level. Bomb from level 1,
  multiplier 2, time stop 3, auto-line 4. With one item it draws nothing, so level 1
  replays stay the same.
- Tokens blink before they despawn and show no countdown, because the digit hid the icon
  for 5 of the 7 s (first shot). The bomb keeps its countdown. `blink_on` is shared.
- The effects label under the HUD bar: "x2 7   STOP 3".
- Evidence:
  - Tests: `level_1_picks_the_bomb_without_a_draw_and_level_4_picks_all_four`,
    `a_gem_and_an_hourglass_start_their_effects_and_an_arrow_its_shot`,
    `the_label_shows_each_effect_with_its_seconds_left`,
    `a_gem_doubles_kill_and_claim_points_while_it_lasts`,
    `a_stop_freezes_enemies_under_gravity_then_gives_back_their_motion`,
    `a_frozen_enemy_still_meets_the_line`,
    `the_shot_follows_the_last_40_units_past_the_free_region`,
    `an_arrow_shoots_the_line_on_to_the_wall_and_claims`,
    `a_shot_that_would_cross_the_line_does_not_fire`. 81 tests pass.
  - Clippy is clean with and without `dev_tools`, and fmt is clean.
  - `just replay`: the hashes are unchanged (seed1 7f4efff7fa02493e, seed2 fc34b9625c975fb3,
    seed5 663c8e56b9e91b4f, seed7 3a318aeeae525404, seed8 d90988d97e91ad2b).
  - `just shot`: the md5 is unchanged (839615a6d1b69a4c0f1cb3eeee7ffffe).
  - `just shot-flow`: a level-2 item spawns, and GameOver clears everything.
  - Visual check: I patched level 1 for a moment to spawn the three tokens and start both
    effects, then restored it (checked with cmp). Shot: `target/shots/s046-items.png`.
- Open points:
  - An auto-line that a claim covers (not touched by the line) only pays its points.
    There is no line left to shoot.
  - Frozen enemies have no tint. Only the label and the stillness show the stop.
    [025] and [047] can judge whether that is enough.
  - Durations, first levels and token size are for [047] (Notes added there).
