# [084] Sounds for the M4 additions

**Status:** open
**Milestone:** M4
**Depends on:** [042], [045], [046], [069]
**Created:** 2026-11-03
**Closed:** -

## Problem
[042] made the sounds of the game as it was. The new parts have none: the Shooter's shot
([045]), the Splitter's hatch ([044]), and the items of [046] and [069] (multiplier, stop
time, auto-line, Star; the Star also changes the music, D18).

## Approach
Once [046] and [069] exist, one ElevenLabs batch as in [042] (`tools/elevenlabs.py sfx`,
three takes each, picked by measurement unless Pedro picks by ear), new `Sfx` variants
triggered where the events happen, rows in `audio-src/sources.md`. The shot is rate-limited
like the bounce if many Shooters fire at once.

## Verification
Each new event logs `audio: audio/sfx/<name>.ogg starts` in a scripted or emulator run;
`cargo test` passes; total SFX size in Notes.

## Notes
- Filed from [045] (2026-11-03): the Shooter fires silently for now.
