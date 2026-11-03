# [041] Audio system - music and SFX channels

**Status:** done
**Milestone:** M4
**Depends on:** [027]
**Created:** 2026-11-02
**Closed:** 2026-11-03

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
- From [026] (2026-11-02): use bevy_new_2d's audio pattern: `Music` and `SoundEffect`
  marker components with bundle functions (`music(handle)`, `sound_effect(handle)`) and
  a volume per category; the module is `src/audio.rs` with `pub(super) fn plugin`.
- Result (2026-11-03): `src/audio.rs` after bevy_new_2d: `Volumes { music: 0.6, sfx: 1 }`
  (a resource for [048]), `AudioAssets` (`LoadResource`), `sound_effect(handle)` (an
  `Add<SoundEffect>` observer sets the start volume, so a muted category never clicks),
  `MusicTrack` set on `OnEnter(Title)` (menu) and `OnEnter(Gameplay)` (game; game over keeps
  it), `switch_music` + `fade_music` crossfade in 1 s on real time (music goes on while
  paused or dying), sinks pause on `AppLifecycle::Suspended` and play on `Running`, a
  button `Activate` plays the tap. `log_started` logs `audio: <path> starts` for checks
  without speakers. A shot run sets both volumes to 0 (it still plays, so the log works).
  - Placeholders: `assets/audio/sfx/tap.ogg`, `music/menu.ogg`, `music/game.ogg`, sine
    tones from ffmpeg (own work, rows in `audio-src/sources.md`); [042] and [043] replace
    them. Homebrew's ffmpeg has no libvorbis: the native `vorbis` encoder needs
    `-strict -2` and is weak; for the real files install `vorbis-tools` (`oggenc`) or
    encode with a libvorbis build. Bevy's default features decode Vorbis; Opus is not
    supported by bevy_audio.
  - Tests: `a_new_track_crossfades_from_the_old_one` (both play at 0.3 to 0.7 halfway,
    the old one despawns), `the_same_track_keeps_playing`,
    `a_sound_effect_starts_at_its_category_volume`.
  - Desktop (shot runs): the flow logs `game.ogg` and `tap.ogg` (RETRY) starting, the
    menu shot logs `menu.ogg`; shots and `just replay` unchanged.
  - Android emulator (debug APK, emulator started with `-no-audio`): `menu.ogg` starts at
    the title, PLAY logs `tap.ogg` then `game.ogg`; HOME and back: no crash, the app runs
    on. The emulator's audio HAL logs `pcm_writei` I/O errors (no host audio), Rust logs
    no audio error. Hearing the crossfade and the suspend pause is left to [025].
  - iOS simulator (`just ios-sim`, Mac output muted for the check): `menu.ogg` starts,
    CoreAudio opens RemoteIO output, no error. simctl cannot tap, so PLAY was not pressed
    there. The simulator plays through the Mac speakers: terminate the app
    (`xcrun simctl terminate booted top.cosas.rix`) before unmuting.
  - Volumes: the start-volume test covers sound effects; `fade_music` writes
    `Volumes.music * gain` to the music sink every frame, so a settings change ([048])
    applies at once.
