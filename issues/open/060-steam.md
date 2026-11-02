# [060] Steam release

**Status:** open
**Milestone:** M6
**Depends on:** [058], [051]
**Created:** 2026-11-02
**Closed:** -

## Problem
Steam needs a Steamworks account and app (with the app fee), a store page in review, and
builds uploaded with SteamPipe.

## Approach
Steamworks app (owner-run), depots per OS uploaded by the release workflow, Steam Deck
compatibility check (touch or gamepad, [057]), optional achievements via a Steamworks
crate only if [012] Q5 asks for them.

## Verification
The Steam build installs from a beta branch and runs on macOS, Windows and (if targeted)
Steam Deck.

## Notes
- **owner-run** account, payment and page review.
