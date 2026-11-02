# [065] Dynamic bodies with no mass or inertia (bomb, arrow)

**Status:** open
**Milestone:** M3
**Depends on:** -
**Created:** 2026-11-02
**Closed:** -

## Problem
Found during [064]: a seed 10 shot run logs
`WARN avian2d::dynamics::rigid_body::mass_properties: Dynamic rigid body 606v0 has no mass
or inertia. This can cause NaN values.` Two spawns can cause it (not yet told apart):
- `items/basic_bomb.rs`: `RigidBody::Dynamic` with `Mass(0.)`.
- `enemies/arrow.rs`: `Collider::polyline` has no area, so its angular inertia is zero
  (`Mass(3.0)` sets the mass only).
Earlier shot logs were filtered, so it is not known since which avian version it shows.

## Approach
Find which entity logs it (`Name` on the arrow, or a spawn log). Give the body real mass
properties: a bomb mass above zero (or a `Sensor` / kinematic body if it must not push),
and for the arrow an `AngularInertia` or a closed collider (`Collider::convex_hull` of the
points), or lock rotation if `ArrowRotation` drives it. Check that it plays the same.

## Verification
`RIX_SEED=10 RIX_SHOT=target/shots/065.png cargo run --features dev_tools 2>&1 | grep -c
"no mass or inertia"` prints 0 (and the same for a few other seeds with a bomb or arrow);
`just shot` matches the previous shot.

## Notes
