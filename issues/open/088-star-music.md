# [088] Star music: two fast tracks while the Star lasts

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
A test: inserting `Star` sets `MusicTrack` to a star track (2A, then 1B, then 2A), removing it
sets the game track again; `every_sound_file_decodes` passes with the new file; `audio-src/sources.md` lists both.

## Notes
- Pedro picked `chiptune-star-2a` on the page (2026-11-03) and wants `chiptune-star-1b` too
  (D26): "I also like 1B alongside 2A so let's have that song too". So two tracks,
  `star_1.ogg` (2A) and `star_2.ogg` (1B); each new Star takes the other one, 2A first. A
  star picked up while one lasts keeps the track that plays. With the soundtrack locked
  ([089]) they play at volume 0, as all music.
- Size: 372 KB and 380 KB (29 s each, -16 LUFS, `oggenc -q 3`); the music is then about
  3.9 MB, inside the 4 MB budget of [043].
