# Territory model spike ([029])

2026-11-02. A spike to choose how Rix keeps the claimed area. The code is in
`docs/research/territory_spike/` (a separate crate:
`cd docs/research/territory_spike && cargo run --release -- <claims> <seed | B>`).

## Today

Each claim is a separate polygon. A line that starts or ends on an older polygon is
spliced onto that outline by a nearest-vertex search (`handle_build_polygon`,
`find_closest_point_index`) and then closed around the wall corners. The git history
shows that this code broke many times. Since [024], the area is an exact scanline union of
the outlines and the collider is a trimesh of the fill.

## The two models

**A: one free-region polygon.** The free region (the part with the enemies) is one simple
polygon. It starts as the playable box. Each line runs from its boundary to its boundary
and cuts it in two. The side without enemies becomes a claimed piece; the other side is
the new free region. (If both sides have enemies, the game's current rule picks one and
eats its enemies.) The cut needs no boolean library: find the start and end positions on
the boundary, then walk the boundary between them (`locate`, `walk`, `split`, about 60
lines). Claimed area = box area - free area (shoelace), exact.

**B: a raster grid (classic Qix).** Cells are free or claimed. Rasterise the line, flood
fill the free cells, and claim the regions that have no enemy.

## Fuzz

A finger-like random walk draws each line: it starts on the free boundary (20 % of starts
from a corner, along the corner bisector), turns slowly, never crosses itself, and ends
where it meets the boundary. Three enemies sit at random free points. The field resets at
75 %. A cut is invalid when a result polygon is not simple, has no positive area, or the
two areas do not add up to the old free area.

**The input rule (A needs it).** A line point closer than 1 unit to the free boundary ends
the line: the line keeps that point and then snaps to the nearest boundary point. The
rule is checked every 0.5 units along each segment. Without the rule, about 3 % of cuts
were invalid: the line ran along a wall, within float noise of it. With the rule, a line
can never touch the boundary except at its two ends, so the cut is always clean. The
player does not see a 1-unit snap (the field is 450 x 800 units).

| Model | Claims | Invalid | Time per claim (desktop release) |
|---|---|---|---|
| A, no input rule | 20,000 x 3 seeds | about 3 % | mean 3-4 µs |
| A, input rule | 20,000 x 8 seeds | **0** | p50 3.4 µs, p99 9 µs, p99.9 25-36 µs |
| B, 1-unit cells | 3,000 | 0 (by construction) | p50 1.3 ms, p99 12 ms |
| B, 2-unit cells | 3,000 | 0 (by construction) | p50 0.35 ms, p99 1.4 ms |

Notes on the numbers:
- A: the free polygon reaches about 450-530 vertices late in a level; the mean line has 57
  points. A few single claims took 3-20 ms in a 20,000-claim run while the next p99.99
  claim took 40 µs: that is OS scheduling, not the cut.
- B: flood fill cost scales with the free cells. A phone is about 3x slower than the
  desktop, so 2-unit cells are about 1 ms per claim (p99 about 4 ms).
- Two fuzz checker bugs were found and fixed on the way (not model bugs): a parallel
  test with an absolute epsilon, and a missing collinear-overlap test. Both showed up
  when a line starts and ends on the same edge.

## Comparison

| | A: free-region polygon | B: grid |
|---|---|---|
| Robust | 0 invalid in 160k claims, with the input rule | yes, by construction |
| Cost on a phone | about 10-100 µs per claim | about 1 ms (2-unit cells) |
| Look | smooth, exact line edges, as today | staircase edges unless smoothed (marching squares) |
| Render | each claimed piece is a simple polygon: the current lyon fill and outline mesh, unchanged | new: a texture or merged meshes, and an outline pass for the neon look |
| Collider | trimesh per piece ([024]), unchanged | new: voxels or marching-squares outlines |
| Area | exact (shoelace) | exact cell count |
| Effort | medium: replace the splice code with `split`, add the input rule | large: new render, collider and outline paths |

## Recommendation

**A.** It keeps the current render and collider paths and the smooth neon edges, and it
replaces the fragile splice code with a small cut that has a fuzz test. The build issue
moves `locate`, `walk`, `split` and the input rule into `src/rix/`, puts the fuzz in a
unit test, and computes progress from the free area. That also ends the box quirk ([029]
Notes): progress will read the real fraction, so the 75 % target may need a retune.
