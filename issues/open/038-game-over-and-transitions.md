# [038] Game over and level transition screens and animations

**Status:** open
**Milestone:** M4
**Depends on:** [026], [030]
**Created:** 2026-11-02
**Closed:** -

## Problem
Death jumps straight to a text screen ("YOU DIED!", score, level, kills, "Tap to play
again."); a level clear only fades the claimed shapes away. todo.md asks for "animations
for next level and game over" and a better game over screen.

## Approach
Death: freeze, flash the line that was hit, then the game-over screen with score, best,
level, kills, seed (if [012] Q4 says so), Retry and Menu buttons. Level clear: a short
celebration (claimed area sweeps, "LEVEL N" banner) before the next level.

## Verification
A manual run shows both animations; the game-over screen buttons work on desktop and the
iOS simulator.

## Notes
- todo.md: "animations for next level and gave over"; "game over screen".
- [012] Q4 (2026-11-02): the game over screen shows the seed, with a share action ([067]).
