# [039] Visible enemy death when eaten

**Status:** open
**Milestone:** M4
**Depends on:** [030]
**Created:** 2026-11-02
**Closed:** -

## Problem
An eaten enemy vanishes under the claim; the pixel explosion appears, but the player does
not see which enemy died or why. todo.md: "when eating an enemy, the enemy eaten should die
in a way that is visible to the player, some animation maybe?"

## Approach
Keep the eaten enemy visible above the claim for a short death animation (flash, shrink,
burst in its own colour), then despawn; per-kind variants later.

## Verification
A seeded scripted run that eats an enemy shows the animation in a screenshot sequence.

## Notes
- todo.md line above.
