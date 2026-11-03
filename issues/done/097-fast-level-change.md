# [097] Fast level change: no banner wait, a press in the change counts

**Status:** done
**Milestone:** M4
**Depends on:** [038]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Pedro, playing on Android (D30): "When transitioning to a new level the "new level screen"
makes it so if you touch the wall its not detected as a touch as the new level is not
ready yet so let's remove that and also make the transition faster/smoother. its very
frustrating, this should be a fast-paced game".

The level change waits 1.2 s for the "LEVEL N" banner ([038]). The input systems are
`PausableSystems`, which run only in `Phase::Playing`, so a touch that starts in the
change is lost: its `Started` event is read by nobody, and the moves after it start no line.

## Approach
- Remove the banner: the next level starts when the claimed shapes finish their fade
  (0.3 s).
- `handle_input_events` also runs in `Phase::LevelTransition`: a press made then waits in
  `InputState::steps`, and the first tick of the next level starts the line from it, on
  the new free region.

## Verification
`cargo test` (a press recorded in the change claims on the next level's region) and a
scripted headless run where a touch that starts in the change draws a claim in level 2;
`just replay` hashes unchanged; `just shot-flow` shows the clear, the death and the retry.

## Notes
- 2026-11-03: done. `level.rs` has no `Banner` any more; `handle_transition_loading` waits
  only for the claimed shapes' 0.3 s fade. `handle_input_events` runs with
  `in_state(IsPaused::Running).or_else(in_state(Phase::LevelTransition))` (Bevy 0.20 has
  no `or`), `handle_input` stays in `PausableSystems`. One system instance in both states,
  so no message is read twice. `cleanup_resources` (on exit of `Playing`) still clears the
  input at the start of the change; the touch that finished the claim was already
  released by the claim, so only a new press carries over.
- Evidence: test `a_press_while_the_level_changes_starts_the_line_in_the_next_level`
  (129 tests). A one-off probe in the flow scene (not committed) sent a touch `Started` at
  the left wall 0.08 s into the change, then moves to the right wall in level 2: the log
  shows "Starting drawing" on level 2's first tick and a claim (1 shape, 1 territory).
  Level 2 now starts 0.38 s after the clear (it was about 1.25 s). `just replay` hashes
  and `just shot` (md5 e170997305a806320c6f9f967cf3cf5b) unchanged; `just shot-flow`
  plays the clear, the death and the retry.
- A press on the HUD's pause button during the change is not dropped (the HUD despawns
  with `Playing`): it lands on the top wall and starts a line there. A tap ends it at
  once, so it draws nothing.
