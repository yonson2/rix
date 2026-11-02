# [034] Pause

**Status:** open
**Milestone:** M3
**Depends on:** [026]
**Created:** 2026-11-02
**Closed:** -

## Problem
`IsPaused` exists and most systems run only in `IsPaused::Running`, but nothing ever enters
`IsPaused::Paused`. Physics keeps running regardless (avian is not paused). A phone call or
a swipe to another app kills the run.

## Approach
A pause button in the HUD and a pause menu (resume, restart, quit to menu); pause on app
suspend ([023]) and on window focus loss on desktop; pause physics time (`Time<Physics>`)
and virtual time together.

## Verification
In a `just shot`-style scripted run: pause mid-drag, wait 2 s, resume: enemy positions did
not change during the pause.

## Notes
