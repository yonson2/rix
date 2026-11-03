# [043] Music

**Status:** done
**Milestone:** M4
**Depends on:** [041], [011]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- 2026-11-03, done. Pedro's picks on the page (`picks/pedro`, 08:28): style chiptune, menu
  `chiptune-menu-1b`, one game track `chiptune-game-1a`. With one game track there is no
  rotation by level: `game.ogg` loops through the run, `menu.ogg` on the title, as the
  placeholders did (`PlaybackSettings::LOOP`, the [041] crossfade). The masters went to
  `audio-src/music/{menu,game}.mp3` with their sidecars (request, take, encode). Encode:
  tail under -50 dB trimmed, gain -2.1 / -2.4 dB to -16.1 LUFS (true peak -4.7 / -4.4
  dBTP), 10 ms fade in, `oggenc -q 3` (110 kbps). Both start at once and end on a 2 s
  decay, so the loop restarts the song.
- Budget: music at most 4 MB in the bundle. Now 3.3 MB (`game.ogg` 2.06 MB for 150 s,
  `menu.ogg` 1.22 MB for 89 s); the debug APK grew from 140.2 to 143.2 MB.
- Credits: `audio-src/sources.md` has a row per track (Suno, D22); the in-game credits
  screen is [052], which lists that sheet.
- Verification: test `the_music_tracks_decode` (now `every_sound_file_decodes`, [087]; the game's decoder reads both files and
  they are not silent); `just shot`/`just shot-menu` load them (shot runs are muted,
  [041]); emulator: the title and a game start with the new files, no audio error in the
  log (the emulator runs with `-no-audio`, so the ear check is Pedro's on a phone, [025]).
- The same page held Pedro's SFX picks by ear: four differ from the shipped takes (tap,
  line_start, line_hum, death); [087] swaps them.

