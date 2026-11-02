# [012] Gate: v1 content scope

**Status:** open
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

## Problem
"More enemies, items, FX" is open-ended. v1 needs a fixed list, or M4 never ends.

## Approach
Questions for Pedro:
1. Enemies for v1: today Ball, Square (teleports), Arrow (chases the finger). Candidates:
   Splitter ([044], spec in the untracked `splitter.md`), Shooter ([045]). Others?
2. Items for v1: today Basic Bomb. Candidates from todo.md: point multiplier, stop time,
   auto-line ([046]). Which?
3. Structure: endless levels (today) or a set of levels with an end? Lives or one death
   ends the run (today: one death)?
4. Seeds: the game seeds every run "so games can be shared" (commit f554584). Show and
   share a seed in v1? A daily seed?
5. Leaderboards or achievements (Game Center, Play Games, Steam) in v1, or later?

## Verification
Every question has a dated answer in `decisions.md`, and this gate closes.

## Notes
- **gate**. Feeds M4.
