# [087] Pedro's sound effect picks by ear

**Status:** open
**Milestone:** M4
**Depends on:** [042], [043]
**Created:** 2026-11-03
**Closed:** -

## Problem
[042] picked each sound effect's take by measurement, not by ear. On the listening page of
[043] (`picks/pedro`, 2026-11-03 08:28) Pedro listened to the three takes of each [042]
sound. The page started on the shipped take; he changed four: `tap` 3 to 1, `line_start`
1 to 3, `line_hum` 1 to 2, `death` 3 to 2. The other five stay.

## Approach
Copy each new take from `audio-src/takes/sfx/` to `audio-src/sfx/NAME.wav`, update its
sidecar (`picked_from`, `picked_by`), and encode it as [042] did (tail trim except the
`line_hum` loop, the sidecar's peak, `oggenc -q 5`). Keep each sound's peak level unless a
take sounds much louder or quieter next to the others (then measure and note it).

## Verification
`audio-src/sfx/{tap,line_start,line_hum,death}.json` name the picked takes; the `.ogg`
files decode (a test, as `the_music_tracks_decode`); `cargo test` passes.

## Notes
