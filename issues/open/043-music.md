# [043] Music

**Status:** open
**Milestone:** M4
**Depends on:** [041], [011]
**Created:** 2026-11-02
**Closed:** -

## Problem
No music. (Seven incompetech mp3s sat untracked in `assets/songs/`; they are gone, D15.)

## Approach
Generate our own tracks, as in Ketania (`tools/gen-music.py`: ElevenLabs, Lyria, Suno),
with a tracked prompt per track. Menu loop plus 2-3 in-game loops, `.ogg` at a
mobile-friendly bitrate.

## Verification
Music plays on menu and in game on desktop and mobile; the bundle size stays within the
budget written in Notes; credits list every track.

## Notes
- [011] Q3 (2026-11-02): own music. The incompetech tracks in `assets/songs/` are not used
  (they went to the Trash, D15).
