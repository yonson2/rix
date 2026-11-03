# [084] Sounds for the M4 additions

**Status:** done
**Milestone:** M4
**Depends on:** [042], [045], [046]
**Created:** 2026-11-03
**Closed:** 2026-11-03

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
- Scope (2026-11-03): the Star does not exist yet, so its sound moved to [069] (note
  there); [084] no longer waits for it.
- Done 2026-11-03. Five sounds from ElevenLabs Sound Effects (`eleven_text_to_sound_v2`,
  Creator plan), three takes each, 138 credits: `shot`, `hatch`, `multiplier`,
  `time_stop`, `auto_line`. Picked by measurement as in [042] (instant start, front-loaded
  energy, short): shot-3, hatch-2 (hatch-1 started 178 ms late), multiplier-2,
  time_stop-2, auto_line-1. Same encode as [042]: trailing trim at -50 dB (no take had a
  silent tail to cut; each ends near -60 dB RMS), peak shot -8 dB, hatch -6, the three
  items -3, `oggenc -q 5`. Pedro can re-pick by ear from `audio-src/takes/sfx/`.
- Hooks: `Sfx::Shot` in `shooter::aim_and_fire` (at most one per 80 ms, the bounce's rate
  limit now covers both: Shooters fire in step), `Sfx::Hatch` in `splitter::hatch`,
  `Sfx::Multiplier`, `Sfx::TimeStop` and `Sfx::AutoLine` in `items::activate_item`. No
  haptic pulse for the new sounds (`haptics::on_sfx`).
- **Total SFX size: 287 KB** (14 `.ogg` files; the five new ones 89 KB).
- Evidence: `cargo test` 94 passed (new `many_bounces_or_shots_at_once_play_one_sound_each`;
  the hatch and item tests check their `Sfx`); clippy clean with and without `dev_tools`;
  `just replay` hashes unchanged; `just shot` md5 unchanged (e170997305a806320c6f9f967cf3cf5b).
  A patched shot run (Shooter on level 1, first shot after 0.02 s, the four other sounds
  triggered on entering `Gameplay`; patch reverted) logged `audio: audio/sfx/shot.ogg starts`,
  `hatch.ogg`, `multiplier.ogg`, `time_stop.ogg` and `auto_line.ogg`. The by-ear check is
  Pedro's.
