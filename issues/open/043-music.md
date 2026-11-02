# [043] Music

**Status:** open
**Milestone:** M4
**Depends on:** [041], [011]
**Created:** 2026-11-02
**Closed:** -

## Problem
No music. Seven incompetech tracks sit untracked in `assets/songs/` (~41 MB of mp3) with a
stub `credits.txt` (D8: untouched until [011] decides).

## Approach
Per [011] Q3: either keep chosen incompetech tracks (CC-BY: credit in game and on the store
pages, re-encode to `.ogg` at a mobile-friendly bitrate), or generate own tracks. Menu
loop plus 2-3 in-game loops.

## Verification
Music plays on menu and in game on desktop and mobile; the bundle size stays within the
budget written in Notes; credits list every track.

## Notes
