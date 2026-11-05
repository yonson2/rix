# [128] The Shooter fires all the time

**Status:** done
**Milestone:** none (balance, a patch release)
**Depends on:** [045], [121]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
Pedro (2026-11-05), after the BULLET HELL challenge: "The shooter feels very weak, because
it takes time to shoot from the moment the player touches the wall, on the shooters only
challenge I could basically survive for an endless amount of time as long as I'm quick
drawing lines."

Today a Shooter fires only while a line is drawn: the first shot 0.5 s into the line,
then one a second. A bullet flies at 140 units a second, so a line that closes within
about a second is safe.

## Approach
Pedro: "What if the shooter shoots at a random direction on its fixed rate but the
direction moves towards the player if a line is being drawn? so the map will always have
shoots going over it".
- The gun fires every `FIRE_SECS` (1 s), with or without a line. Each Shooter's timer
  starts at a random phase, so turrets do not fire in step.
- With no line, each shot picks a new random aim; the barrel turns to it.
- While a line is drawn, the aim is the finger.
- The barrel turns at `TURN_SPEED` (half a turn a second) and fires where it points.
- Random draws come from `GameRng` in `FixedUpdate`, over the guns in `by_place` order.
- The tip and the wiki say "IT FIRES ALL THE TIME".

## Verification
`cargo test` (the gun fires every second with or without a line; it turns at most
`TURN_SPEED`; idle shots go in different directions); both clippy runs; `just replay`
passes (only the shooters-only seed 12G2-1000-0000-2 gets a new hash in
`tools/replay-hashes.txt`); the `just shot` md5 stays e170997305a806320c6f9f967cf3cf5b; a
BULLET HELL shot shows bullets in flight; `just store-art` still runs through.

## Notes
- 2026-11-05: done on `fix/128-shooter`. `Gun` holds a repeating timer (a random phase at
  spawn), `heading` (where the barrel points), `idle` (the random way) and `finger`.
  `TURN_SPEED` is half a turn a second, so an idle barrel reaches any new way before the
  next shot. Spawn draws two more numbers (angle and phase), so a level with a Shooter
  plays another way for the same seed.
- Evidence: 202 tests pass (new: `the_gun_fires_every_second_with_or_without_a_line`,
  `with_no_line_each_shot_goes_another_way`,
  `while_a_line_is_drawn_the_barrel_turns_to_the_finger`); both clippy runs and `cargo fmt
  --check` pass; the `just shot` md5 stays e170997305a806320c6f9f967cf3cf5b; `just replay`
  passes, and only 1243064965410062338 (12G2-1000-0000-2) has a new hash, f72a2da7b6c7ea60
  (the same at 30, 60 and 120 fps). A shot of that seed at level 5 shows five bullets in
  flight; the old build shows none.
- Store art: store shot 2 (seed 13, level 9, line set 0) now dies before its shot. Sweep
  (line sets 0 to 2, seeds 1 to 30, levels 8 to 10, both sizes): at level 9, set 0 lives
  only when its third line starts on a claim (no line in the picture). New shot 2: seed 3,
  level 8, set 0 (STAR on, a Shooter and its bullets, two arrows, a bomb). Set 1 seed 1
  level 9 also fits, but it has shot 3's line set and claimed shape. `just store-art` runs
  through.
- Pedro played it on his phone (2026-11-05): "Yeah, that is perfect now". No tuning; it
  merges as 1.3.2.
