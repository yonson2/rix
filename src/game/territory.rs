//! The free region ([074], model A of [029], D19): one simple counter-clockwise polygon, the
//! part of the field where the enemies are. Each line runs from its boundary to its
//! boundary and cuts it in two: one side becomes a claimed piece, the other the new free
//! region.

use bevy::prelude::*;

use super::{state::Phase, wall::Field};

/// The input rule: a line point closer than this to the free boundary ends the line.
const EPS: f32 = 1.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<FreeRegion>()
        .add_systems(OnEnter(Phase::Playing), reset_free_region);
}

#[derive(Resource, Clone, Debug, Deref)]
pub struct FreeRegion(pub Vec<Vec2>);

impl Default for FreeRegion {
    fn default() -> Self {
        Self::of(&Field::default())
    }
}

impl FreeRegion {
    /// The playable box of `field`, inside the walls.
    pub fn of(field: &Field) -> Self {
        let c = field.inner_corners();
        Self(vec![c.bottom_left, c.bottom_right, c.top_right, c.top_left])
    }

    /// The claimed part of the box of `field`, exact.
    pub fn claimed_fraction(&self, field: &Field) -> f32 {
        1. - signed_area(self) / signed_area(&Self::of(field))
    }
}

fn reset_free_region(mut free: ResMut<FreeRegion>, field: Res<Field>) {
    *free = FreeRegion::of(&field);
}

/// The shoelace formula: positive for a counter-clockwise polygon.
pub fn signed_area(p: &[Vec2]) -> f32 {
    let mut s = 0.;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        s += a.x * b.y - b.x * a.y;
    }
    s / 2.
}

/// The nearest boundary point to `p`: (edge index, t along the edge, point).
pub fn locate(poly: &[Vec2], p: Vec2) -> (usize, f32, Vec2) {
    let mut best = (0, 0., poly[0], f32::MAX);
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let t = ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0., 1.);
        let q = a + (b - a) * t;
        let d = q.distance_squared(p);
        if d < best.3 {
            best = (i, t, q, d);
        }
    }
    (best.0, best.1, best.2)
}

/// The vertices met walking the boundary forward from position `from` to position `to`.
fn walk(poly: &[Vec2], from: (usize, f32), to: (usize, f32)) -> Vec<Vec2> {
    let n = poly.len();
    let mut out = vec![];
    if from.0 == to.0 && from.1 <= to.1 {
        return out;
    }
    let mut k = (from.0 + 1) % n;
    loop {
        out.push(poly[k]);
        if k == to.0 {
            break;
        }
        k = (k + 1) % n;
    }
    out
}

/// Cuts a simple counter-clockwise polygon with a path from boundary to boundary. Both
/// sides are counter-clockwise.
pub fn split(poly: &[Vec2], path: &[Vec2]) -> [Vec<Vec2>; 2] {
    let (i, ti, a) = locate(poly, path[0]);
    let (j, tj, b) = locate(poly, *path.last().unwrap());
    let mut inner = path.to_vec();
    inner[0] = a;
    *inner.last_mut().unwrap() = b;
    let mut one = inner.clone();
    one.extend(walk(poly, (j, tj), (i, ti)));
    let mut two: Vec<Vec2> = inner.iter().rev().copied().collect();
    two.extend(walk(poly, (i, ti), (j, tj)));
    [dedup(one), dedup(two)]
}

fn dedup(mut p: Vec<Vec2>) -> Vec<Vec2> {
    p.dedup_by(|a, b| a.distance(*b) < 1e-4);
    while p.len() > 1 && p[0].distance(*p.last().unwrap()) < 1e-4 {
        p.pop();
    }
    p
}

/// (t on ab, u on cd) of a crossing; parallel segments do not cross.
fn cross(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<(f32, f32)> {
    let r = b - a;
    let s = d - c;
    let den = r.perp_dot(s);
    if den.abs() < 1e-6 * r.length() * s.length() {
        return None;
    }
    let t = (c - a).perp_dot(s) / den;
    let u = (c - a).perp_dot(r) / den;
    ((0. ..=1.).contains(&t) && (0. ..=1.).contains(&u)).then_some((t, u))
}

/// The first point after `p` where the segment `p`-`q` crosses the boundary.
fn first_hit(poly: &[Vec2], p: Vec2, q: Vec2) -> Option<Vec2> {
    let mut best: Option<(f32, Vec2)> = None;
    for i in 0..poly.len() {
        if let Some((t, _)) = cross(p, q, poly[i], poly[(i + 1) % poly.len()])
            && t > 1e-5
            && best.is_none_or(|(bt, _)| t < bt)
        {
            best = Some((t, p + (q - p) * t));
        }
    }
    best.map(|b| b.1)
}

/// The input rule ([029]): where the line segment `p`-`q` ends the line, as the point the
/// line keeps and its snap on the boundary. The first point within `EPS` of the boundary
/// ends it, sampled every `EPS / 2`; a segment that starts that close (the first one)
/// ends only where it crosses the boundary. So a line touches the boundary only at its
/// two ends, and the cut is clean.
pub fn line_end(poly: &[Vec2], p: Vec2, q: Vec2) -> Option<(Vec2, Vec2)> {
    let crossing = first_hit(poly, p, q);
    if locate(poly, p).2.distance(p) < EPS {
        return crossing.map(|c| (c, c));
    }
    let limit = crossing.unwrap_or(q);
    let steps = ((limit - p).length() / (EPS * 0.5)).ceil().max(1.) as i32;
    (1..=steps)
        .map(|k| {
            let r = p.lerp(limit, k as f32 / steps as f32);
            (r, locate(poly, r).2)
        })
        .find(|(r, b)| b.distance(*r) < EPS)
        .or(crossing.map(|c| (c, c)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::polygon::is_point_in_polygon;

    fn box_region() -> Vec<Vec2> {
        FreeRegion::default().0
    }

    /// Parallel segments on one line that share more than a point.
    fn overlap(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
        let r = b - a;
        let s = d - c;
        if r.perp_dot(s).abs() >= 1e-6 * r.length() * s.length()
            || r.normalize().perp_dot(c - a).abs() > 1e-3
        {
            return false;
        }
        let len = r.length();
        let (u, v) = (r.dot(c - a) / len, r.dot(d - a) / len);
        u.max(v).min(len) - u.min(v).max(0.) > 1e-3
    }

    /// No two non-adjacent edges touch.
    fn is_simple(p: &[Vec2]) -> bool {
        let n = p.len();
        if n < 3 {
            return false;
        }
        for i in 0..n {
            for j in i + 2..n {
                if i == 0 && j == n - 1 {
                    continue;
                }
                let (a, b, c, d) = (p[i], p[(i + 1) % n], p[j], p[(j + 1) % n]);
                if cross(a, b, c, d).is_some() || overlap(a, b, c, d) {
                    return false;
                }
            }
        }
        true
    }

    fn crosses_path(path: &[Vec2], p: Vec2, q: Vec2) -> bool {
        path.windows(2)
            .take(path.len().saturating_sub(2))
            .any(|w| cross(w[0], w[1], p, q).is_some())
    }

    fn random_inside(rng: &mut fastrand::Rng, poly: &[Vec2]) -> Vec2 {
        let c = Field::default().inner_corners();
        loop {
            let p = c.bottom_left + Vec2::new(rng.f32(), rng.f32()) * (c.top_right - c.bottom_left);
            if is_point_in_polygon(&p, poly) {
                return p;
            }
        }
    }

    /// A finger-like line: it starts on the boundary (20 % of starts from a corner, along
    /// the corner bisector), turns slowly, never crosses itself and ends by the input rule.
    fn random_path(rng: &mut fastrand::Rng, free: &[Vec2]) -> Option<Vec<Vec2>> {
        let e = rng.usize(..free.len());
        let (a, b) = (free[e], free[(e + 1) % free.len()]);
        let t = if rng.f32() < 0.2 {
            0.
        } else {
            0.05 + rng.f32() * 0.9
        };
        let start = a + (b - a) * t;
        let mut inward = (b - a).perp().normalize();
        if t == 0. {
            let prev = free[(e + free.len() - 1) % free.len()];
            inward = (inward + (a - prev).perp().normalize()).normalize_or_zero();
        }
        let mut path = vec![start];
        let mut cur = start + inward * 1.5;
        if !is_point_in_polygon(&cur, free) || first_hit(free, start + inward * 0.01, cur).is_some()
        {
            return None;
        }
        path.push(cur);
        let mut heading = inward.to_angle() + rng.f32() * 1.2 - 0.6;
        let target = random_inside(rng, free);
        for step in 0..2000 {
            heading += rng.f32() * 0.5 - 0.25;
            if step > 60 {
                heading += ((target - cur).to_angle() - heading).sin() * 0.3;
            }
            let next = cur + Vec2::from_angle(heading) * (2. + rng.f32() * 6.);
            if crosses_path(&path, cur, next) {
                heading += std::f32::consts::FRAC_PI_2;
                continue;
            }
            if let Some((last, end)) = line_end(free, cur, next) {
                if crosses_path(&path, cur, last) || crosses_path(&path, last, end) {
                    return None;
                }
                path.push(last);
                path.push(end);
                return Some(dedup(path));
            }
            path.push(next);
            cur = next;
        }
        None
    }

    /// The [029] fuzz: lines cut the free region until 75 % is claimed, then the field
    /// resets. The side with no enemy is claimed; three enemies stay in the free region.
    #[test]
    fn fuzzed_cuts_give_two_simple_pieces_that_add_up() {
        let mut rng = fastrand::Rng::with_seed(29);
        let box_area = signed_area(&box_region());
        let mut free = box_region();
        let mut enemies = vec![];
        let (mut claims, mut levels, mut fails) = (0, 0, 0);
        while claims < 2_000 {
            while enemies.len() < 3 {
                enemies.push(random_inside(&mut rng, &free));
            }
            let Some(path) = random_path(&mut rng, &free) else {
                fails += 1;
                assert!(fails < 100_000, "no line fits at {} vertices", free.len());
                continue;
            };
            fails = 0;
            let [one, two] = split(&free, &path);
            let (a1, a2) = (signed_area(&one), signed_area(&two));
            assert!(
                is_simple(&one)
                    && is_simple(&two)
                    && a1 > 0.
                    && a2 > 0.
                    && (a1 + a2 - signed_area(&free)).abs() < 1e-3 * box_area,
                "claim {claims}: invalid cut, areas {a1} {a2}"
            );
            claims += 1;
            let kept = if enemies.iter().any(|e| is_point_in_polygon(e, &one)) {
                one
            } else {
                two
            };
            enemies.retain(|e| is_point_in_polygon(e, &kept));
            free = kept;
            if 1. - signed_area(&free) / box_area >= 0.75 {
                levels += 1;
                free = box_region();
                enemies.clear();
            }
        }
        println!("fuzz: {claims} claims, {levels} levels");
        assert!(levels > 10, "the fuzz clears levels: {levels}");
    }

    /// The box with a claimed strip along the bottom wall: its top edge runs at y = -300.
    fn after_bottom_strip() -> Vec<Vec2> {
        let path = [Vec2::new(-190., -300.), Vec2::new(190., -300.)];
        let [rest, strip] = split(&box_region(), &path);
        assert_eq!(signed_area(&strip), 380. * 65.);
        rest
    }

    #[test]
    fn a_line_from_claimed_edge_to_claimed_edge_cuts_a_notch() {
        let free = after_bottom_strip();
        let path = [
            Vec2::new(-100., -300.),
            Vec2::new(-100., -200.),
            Vec2::new(100., -200.),
            Vec2::new(100., -300.),
        ];
        let [rest, notch] = split(&free, &path);
        assert_eq!(signed_area(&notch), 200. * 100.);
        assert!(is_simple(&notch) && is_simple(&rest));
        assert_eq!(signed_area(&notch) + signed_area(&rest), signed_area(&free));
        assert!(is_point_in_polygon(&Vec2::new(0., -250.), &notch));
        assert!(is_point_in_polygon(&Vec2::new(0., 0.), &rest));
        assert!(!is_point_in_polygon(&Vec2::new(0., -250.), &rest));
    }

    #[test]
    fn a_line_from_a_wall_to_a_claimed_edge_cuts_a_corner() {
        let free = after_bottom_strip();
        let path = [
            Vec2::new(-190., -100.),
            Vec2::new(-50., -100.),
            Vec2::new(-50., -300.),
        ];
        let [rest, corner] = split(&free, &path);
        assert!((signed_area(&corner) - 140. * 200.).abs() < 0.1);
        assert!(is_simple(&corner) && is_simple(&rest));
        assert!(is_point_in_polygon(&Vec2::new(-120., -200.), &corner));
        assert!(is_point_in_polygon(&Vec2::new(100., -200.), &rest));
    }

    #[test]
    fn line_end_keeps_the_point_before_the_boundary_and_snaps_onto_it() {
        let free = after_bottom_strip();
        let (kept, snap) = line_end(&free, Vec2::new(100., -250.), Vec2::new(100., -350.)).unwrap();
        assert!(kept.y > -300. && kept.y - -300. < EPS, "kept {kept}");
        assert!(snap.distance(Vec2::new(100., -300.)) < 1e-3, "snap {snap}");
        assert_eq!(
            line_end(&free, Vec2::new(100., -250.), Vec2::new(100., -280.)),
            None
        );
    }

    #[test]
    fn line_end_lets_the_first_segment_leave_the_boundary() {
        let free = after_bottom_strip();
        let start = Vec2::new(0., -300.);
        assert_eq!(line_end(&free, start, Vec2::new(0., -200.)), None);
        let (kept, snap) = line_end(&free, start, Vec2::new(0., 400.)).unwrap();
        assert!(kept == snap && snap.distance(Vec2::new(0., 335.)) < 1e-3);
    }

    #[test]
    fn claimed_fraction_is_exact() {
        let free = |p: Vec<Vec2>| FreeRegion(p).claimed_fraction(&Field::default());
        assert_eq!(free(box_region()), 0.);
        let half = split(
            &box_region(),
            &[Vec2::new(-190., -15.), Vec2::new(190., -15.)],
        );
        assert_eq!(free(half[0].clone()), 0.5);
        assert_eq!(free(half[1].clone()), 0.5);
        assert_eq!(free(vec![]), 1., "a fully claimed box");
    }
}
