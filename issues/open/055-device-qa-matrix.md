# [055] Device QA matrix

**Status:** open
**Milestone:** M5
**Depends on:** [053], [054]
**Created:** 2026-11-02
**Closed:** -

## Problem
Release needs one recorded pass over the devices and flows that matter.

## Approach
A matrix in Notes: devices (small and large iPhone, an iPad if supported, a low-end and a
high-end Android) × flows (first run and tutorial, full run to game over, pause and
resume, background, settings kept, audio, rotation if allowed). Failures become bug
issues.

## Verification
Every cell of the matrix is filled with pass or a linked bug issue.

## Notes
- **owner-run**.
- From [025] (2026-11-03, D29): no iPhone ran the game yet. Include the iOS checks from [025]'s Notes: title tap and slide-off, home indicator swipe, haptics, the share sheet with the death picture, the HUD pause button, 120 Hz smoothness.
