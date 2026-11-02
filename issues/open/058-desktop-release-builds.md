# [058] Desktop release builds and CI release workflow

**Status:** open
**Milestone:** M6
**Depends on:** [008], [057]
**Created:** 2026-11-02
**Closed:** -

## Problem
No packaged desktop build exists for macOS, Windows or Linux.

## Approach
A tag-triggered GitHub Actions release workflow (bevy_new_2d's release workflow as the
base, [026]): macOS universal `.app` (signed and notarized: needs Pedro's Developer ID),
Windows `.zip` (exe + assets), Linux `.tar.gz`; web build too if [010] Q7 says yes.

## Verification
A test tag produces downloadable artifacts for all three platforms; each starts and plays
on its OS.

## Notes
