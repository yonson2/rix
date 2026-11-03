# [043] Music

**Status:** active
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
- 2026-11-03: Pedro chose Suno over Eleven Music and checked Suno's licence himself (D22).
  `tools/suno.py` (the Suno part of Ketania's `gen-music.py`) and `audio-src/music/briefs.json`
  made four styles (synthwave, chiptune, darksynth, minimal techno), each a 150 s game brief
  and a 90 s menu brief, two takes per call: 8 calls, 64 EvoLink credits, MP3 about 185 kbps.
  Takes in `audio-src/takes/music/` (gitignored). Pedro picks the style and the tracks by ear
  on the listening page https://claude.ai/artifact/HNtjXYWjDVAGvQivtAQGYG (picks save to its
  database, `picks/pedro`). Plan after the pick: the game tracks rotate by level
  (`(level - 1) % n`, on each `OnEnter(Phase::Playing)`), the menu track stays on `Title`.
