# [085] Settings from the pause menu

**Status:** open
**Milestone:** M4
**Depends on:** [048], [034]
**Created:** 2026-11-03
**Closed:** -

## Problem
[048] asks for the settings screen from the menu and from the pause menu. The pause
overlay ([034]) has no buttons: a tap, a click or Escape anywhere resumes the run. A
SETTINGS button there needs a new pause design, and the settings screen is a `Screen`
state: going to it from a run would end the run.

## Approach
Give the pause overlay buttons (RESUME, SETTINGS, maybe MENU), so a tap outside them
does nothing or resumes. Show the settings rows from `screens::settings` as a panel inside
`IsPaused::Paused` (a sub-state or a flag), not as `Screen::Settings`, so the run stays.
Android Back and Escape keep their [079] and [034] behaviour.

## Verification
In a paused run, open the settings, change one, go back and resume: the run goes on and
the setting holds (`just shot-flow` census before and after; a test of the pause input).

## Notes
- Split from [048] on 2026-11-03. Pedro decides the pause layout (does a tap outside the
  buttons still resume?).
