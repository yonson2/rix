# [087] Pedro's sound effect picks by ear

**Status:** done
**Milestone:** M4
**Depends on:** [042], [043]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
files decode (test `every_sound_file_decodes`); `cargo test` passes.

## Notes
- 2026-11-03, done. The four takes copied to `audio-src/sfx/` with `picked_from` and
  `picked_by` in their sidecars; encoded with the [042] recipe at each sound's peak (tap
  -8, line_start -6, line_hum -16 untrimmed, death -2 dB), the level the page played them
  at. `line_start-3` peaks at 0 dB in the take (likely clipped at the source); at -6 dB
  it sounds as Pedro heard it. The four files went from 101 to 91 KB; SFX total 277 KB.
  `audio-src/sources.md` notes the re-pick.
- Verification: the four sidecars name the new takes; test `every_sound_file_decodes`
  (every music and SFX `.ogg` decodes with the game's decoder and is not silent); 106
  tests pass, clippy clean with and without `dev_tools`.
