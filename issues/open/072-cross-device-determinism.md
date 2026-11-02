# [072] Same seed, same run on every device and frame rate

**Status:** open
**Milestone:** M3
**Depends on:** [019], [026], [028]
**Created:** 2026-11-02
**Closed:** -

## Problem
D10 Q5 wants a seed to give the same game on any device, and [067] shares seeds. Since
[019] the field is the same everywhere, but the run can still drift: gameplay systems in
`Update` step by the frame time (square teleports, arrow chase, timers that draw from
`LevelInfo.rng`), so a 120 Hz phone and a 60 Hz one diverge; avian's float results may
differ between CPUs (arm64 phones, x86 web hosts) without its `enhanced-determinism`
feature; system order is not fixed ([026]).

## Approach
Move gameplay that moves bodies or draws from the RNG into `FixedUpdate`; turn on avian's
`enhanced-determinism` (check its cost on a phone, [024]); after [026] fixes the system
order, add a headless replay test: a seed and a scripted input run at two frame rates
(and on two targets in CI if cheap) gives the same hash of enemy positions and claimed
area after N seconds.

## Verification
The replay test passes: seed 1 with the scripted line at 30 and 60 frames per second ends
with the same state hash.

## Notes
- Found in [019] (2026-11-02): the field and the input no longer depend on the screen;
  this issue covers the rest of "same seed, same game".
