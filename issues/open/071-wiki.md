# [071] Wiki - enemies and items explained

**Status:** open
**Milestone:** M4
**Depends on:** [036], [070]
**Created:** 2026-11-02
**Closed:** -

## Problem
Pedro's idea (2026-11-02, D18): a wiki section in the menu that explains each enemy and
each item, so a player can look up what something does.

## Approach
A menu entry ([036]) that opens a list: one card per enemy and item with its look (the
live shape or sprite), its name and one or two lines on what it does. Kinds not yet met
([070]) show as locked ("???"). The text lives next to the spawn data ([032]) so one
place names and describes each kind.

## Verification
The menu opens the wiki; every enemy and item kind in the spawn table has a card; a kind
not met yet shows locked (`just shot-menu` with a wiki scene, and a test that every kind
has a description).

## Notes
