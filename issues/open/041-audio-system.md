# [041] Audio system - music and SFX channels

**Status:** open
**Milestone:** M4
**Depends on:** [027]
**Created:** 2026-11-02
**Closed:** -

## Problem
The game is silent: no audio code, no sounds in `assets/`.

## Approach
Music and SFX as separate categories with their own volume (bevy_new_2d's audio module,
[026]); music crossfade between menu and game; stop on suspend ([023]); volume in
settings ([048]). Check `bevy_audio` on iOS and Android (cpal/oboe) before considering
another backend.

## Verification
A test sound and a test track play on desktop, the iOS simulator and the Android emulator;
the volume settings change them.

## Notes
- [023] (2026-11-02): pause the music and SFX when the app suspends (`AppLifecycle`, as Bevy's mobile example does) and resume them with the game ([034] pauses on suspend).
