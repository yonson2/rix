# [100] Store preview video

**Status:** done
**Milestone:** M5
**Depends on:** [051]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
A short video sells a game better than stills. The App Store takes app previews (15 to
30 s, per device class) and Play takes a YouTube link. Both are optional: the stores
accept a listing without one. [051] made the stills only.

## Approach
Record a scripted run from the shot harness ([006]): save a frame per tick (or pipe the
frames to ffmpeg), at each store's preview size, with the game's sound. A longer script
than the store scene: several lines, a level clear, an item. Check each store's current
preview spec first.

## Verification
A video per store that the store's spec accepts (size, length, format), made by one
command from the repo.

## Notes
- Filed from [051] (2026-11-03): the stills are done; the video is optional and needs a
  frame recorder in the harness, so it is its own issue.
- Dropped 2026-11-03 (D32): Pedro, asked whether he wants a preview video: "No".
