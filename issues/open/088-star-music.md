# [088] Star music: a fast track while the Star lasts

**Status:** open
**Milestone:** M4
**Depends on:** [069], [043]
**Created:** 2026-11-03
**Closed:** -

## Problem
[069] asks for a music change while the Star lasts, as in Mario Kart. D25: Pedro picks the
music by ear; the agent does not pick for him. So the swap waits for his pick.

## Approach
Four Suno takes in Pedro's chiptune style (brief `chiptune-star` in
`audio-src/music/briefs.json`, about 30 s, 172 BPM, C major) are on the listening page
https://claude.ai/artifact/HNtjXYWjDVAGvQivtAQGYG ("New: the Star track"; the pick saves to
`picks/pedro`, field `star`). After the pick: encode it as [043] did (`audio-src/music/star.mp3`
and sidecar, -16 LUFS, `oggenc -q 3`, a row in `audio-src/sources.md`), add `star` to
`AudioAssets`, and set `MusicTrack` to it when the `Star` resource appears and back to the
game track when it goes (the [041] crossfade does the rest).

## Verification
A test: inserting `Star` sets `MusicTrack` to the star track, removing it sets the game track
again; `every_sound_file_decodes` passes with the new file; `audio-src/sources.md` lists it.

## Notes
