# [039] Visible enemy death when eaten

**Status:** done
**Milestone:** M4
**Depends on:** [030]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
An eaten enemy vanishes under the claim; the pixel explosion appears, but the player does
not see which enemy died or why. todo.md: "when eating an enemy, the enemy eaten should die
in a way that is visible to the player, some animation maybe?"

## Approach
Keep the eaten enemy visible above the claim for a short death animation (flash, shrink,
burst in its own colour), then despawn; per-kind variants later.

## Verification
`cargo test` (corpse spawn and look) passes; a seeded scripted run with a kill (seed 24:
the line sets off the bomb, which kills the square) shows the animation in a screenshot
sequence.

## Notes
- todo.md line above.
- Done 2026-11-03: `on_despawn_enemy` spawns a `Corpse`, a copy of the enemy's `Mesh2d`
  and `ColorMaterial` (ball, square) or `Shape` (arrow) at z 1.5 (above the claim and
  the enemies, under the pixel explosion). `corpse_look`: it pops to 1.3 times its size in
  `NEON_WHITE` over the first quarter of 0.4 s, then shrinks to nothing in its own
  colour. `DespawnOnExit(Screen::Gameplay)` and a non-pausable `Update` system, so the
  corpse of the claim that clears the level plays during the transition.
- Not done: a burst in the enemy's own colour (the pixel explosion shader has no colour
  input) and per-kind variants; both fit the FX pass ([040]).
- Evidence: the scripted line eats nothing at level 1 for seeds 1 to 40 (a claim eats only
  when both sides hold enemies). Seed 24 kills through the bomb: shots at 1.10, 1.18,
  1.26, 1.34, 1.42 and 1.55 s show the square, its white flash, the shrinking green
  corpse above the claim and the "LEVEL 2" banner, then only the explosion. Default
  `just shot` md5 unchanged; `just replay` passes.
