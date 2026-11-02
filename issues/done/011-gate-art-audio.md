# [011] Gate: Art and audio direction, asset sources and licences

**Status:** done
**Milestone:** M0
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
"Generate assets for the game" needs a direction and sources before any work. Two current
assets also carry a licence risk for a store release.

## Approach
Questions for Pedro:
1. Visual identity: keep the neon-vector-on-black look with bloom and the CRT post-process
   (`src/colors.rs`, `assets/shaders/crt_shader.wgsl`)? Any reference games?
2. Font: `assets/fonts/main.ttf` is named "balatro", copyright "localthunk 2023", and its
   name table says Open Font License. It looks taken from the game Balatro. Keep it after a
   licence check, or pick a font with a clear OFL source (e.g. from Google Fonts)?
3. Music: keep the untracked incompetech tracks in `assets/songs/` (Kevin MacLeod, CC-BY:
   needs credit in game and store page; ~41 MB of mp3, too big for a mobile bundle as is),
   or generate our own (ketania's `tools/gen-music.py`: ElevenLabs, Lyria, Suno)?
4. SFX: generate with ElevenLabs (ketania's `tools/elevenlabs.py`), a synth like sfxr, or
   another source?
5. Images: the game is procedural shapes; the only raster is `assets/items/bomb_basic.png`
   (100x100, source unknown). Procedural icons for items, or generated art (PixelLab,
   Midjourney via EvoLink as in ketania)?
6. App icon: `feat/ios` has a 1024 px `icon.png`. Keep that design?

## Verification
Every question has a dated answer in `decisions.md`, and this gate closes.

## Notes
- **gate**. Feeds [041]-[043], [050], [051], [052].
- Answered by Pedro on 2026-11-02: `decisions.md`, "Gate answers". The answers went into
  [036], [040], [042], [043], [046], [050]-[052].
