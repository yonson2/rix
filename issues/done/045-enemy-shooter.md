# [045] New enemy - Shooter

**Status:** done
**Milestone:** M4
**Depends on:** [032], [012]
**Created:** 2026-11-02
**Closed:** 2026-11-03

## Problem
todo.md: "The Shooter - fixed position but shoots small bullets that travel slowly towards
the player path".

## Approach
A static enemy that fires slow bullets at the current line while the player draws;
bullets kill on contact with the line, die on walls and claims. Design confirmed in [012].

## Verification
A seeded scripted drag near a Shooter dies to a bullet (scripted run or test).

## Notes
- todo.md line above.
- [012] Q1 (2026-11-02): the Shooter is in v1.
- Done 2026-11-03. `src/game/enemies/shooter.rs`: a hexagon turret with a barrel (orange,
  new `ORANGE` / `NEON_ORANGE`), from level 3, so level 1 replays stay the same (`just
  replay` hashes and `just shot` image unchanged). Dynamic with `LockedAxes::ALL_LOCKED`:
  a static body would never meet the line (static colliders get no contacts with each
  other). It aims at the latest `InputPosition` and fires while `UserPath::is_drawing`.
- Bullets: `Sensor` dynamic bodies on the new `GameLayer::Bullet` (the player layer now
  lists it). They are not `Enemy`: a claim must not eat them (points) and they must not
  decide which side a claim takes. They pass enemies and items.
- Verified: `a_bullet_kills_on_the_line_and_stops_on_a_wall` (avian test app: a bullet
  passes an enemy, hits a `TempLineSegment` and writes one `Die`; a wall drops it with no
  death) and `the_gun_fires_half_a_second_into_a_line_then_every_second`. Temporary
  level-1 patches (not committed) drew the turret aiming at the line and, with fast fire,
  bullets in flight (`target/shots/s045-*.png`).
- No shot sound yet: [084].
