//! [029] spike: fuzz two territory models with random lines drawn like a finger would.
//! A: one free-region polygon, cut by each line. B: a raster grid with flood fill.

use glam::Vec2;
use std::time::{Duration, Instant};

/// The playable box inside the walls (450 - 2 * 35 wide, 800 - 35 - 65 tall).
const BOX: Vec2 = Vec2::new(380., 700.);
const TARGET: f32 = 0.75;
const ENEMIES: usize = 3;
/// Model A: how close to the boundary a point ends the line.
const EPS: f32 = 1.;

fn signed_area(p: &[Vec2]) -> f32 {
    let mut s = 0.;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        s += a.x * b.y - b.x * a.y;
    }
    s / 2.
}

fn point_in(poly: &[Vec2], p: Vec2) -> bool {
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (b.x - a.x) * (p.y - a.y) / (b.y - a.y) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// (t on ab, u on cd) of a proper crossing.
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

fn dedup(mut p: Vec<Vec2>) -> Vec<Vec2> {
    p.dedup_by(|a, b| a.distance(*b) < 1e-4);
    while p.len() > 1 && p[0].distance(*p.last().unwrap()) < 1e-4 {
        p.pop();
    }
    p
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
            if cross(p[i], p[(i + 1) % n], p[j], p[(j + 1) % n]).is_some()
                || overlap(p[i], p[(i + 1) % n], p[j], p[(j + 1) % n])
            {
                return false;
            }
        }
    }
    true
}

// ---------------------------------------------------------------- Model A

/// The nearest boundary point: (edge index, t along the edge, point).
fn locate(poly: &[Vec2], p: Vec2) -> (usize, f32, Vec2) {
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

/// Cuts a simple counter-clockwise polygon with a path from boundary to boundary.
fn split(poly: &[Vec2], path: &[Vec2]) -> [Vec<Vec2>; 2] {
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

fn crosses_path(path: &[Vec2], p: Vec2, q: Vec2) -> bool {
    path.windows(2)
        .take(path.len().saturating_sub(2))
        .any(|w| cross(w[0], w[1], p, q).is_some())
}

/// A finger-like random walk from a boundary point until it meets the boundary again.
fn random_path(
    rng: &mut fastrand::Rng,
    inside: impl Fn(Vec2) -> bool,
    hit: impl Fn(Vec2, Vec2) -> Option<(Vec2, Vec2)>,
    start: Vec2,
    inward: Vec2,
) -> Option<Vec<Vec2>> {
    let mut path = vec![start];
    let mut cur = start + inward * 1.5;
    if !inside(cur) || hit(start + inward * 0.01, cur).is_some() {
        return None;
    }
    path.push(cur);
    let mut heading = inward.y.atan2(inward.x) + rng.f32() * 1.2 - 0.6;
    let target = Vec2::new(rng.f32() - 0.5, rng.f32() - 0.5) * BOX;
    for step in 0..2000 {
        heading += rng.f32() * 0.5 - 0.25;
        if step > 60 {
            let want = (target - cur).to_angle();
            heading += (want - heading).sin() * 0.3;
        }
        let next = cur + Vec2::from_angle(heading) * (2. + rng.f32() * 6.);
        if crosses_path(&path, cur, next) {
            heading += std::f32::consts::FRAC_PI_2;
            continue;
        }
        if let Some((last, end)) = hit(cur, next) {
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

fn random_inside(rng: &mut fastrand::Rng, inside: impl Fn(Vec2) -> bool) -> Vec2 {
    loop {
        let p = Vec2::new(rng.f32() - 0.5, rng.f32() - 0.5) * BOX;
        if inside(p) {
            return p;
        }
    }
}

#[derive(Default)]
struct Stats {
    claims: u32,
    levels: u32,
    invalid: u32,
    time: Duration,
    max_time: Duration,
    max_vertices: usize,
    path_points: usize,
    stalls: u32,
    times: Vec<Duration>,
}

fn model_a(seed: u64, claims: u32) -> Stats {
    let mut rng = fastrand::Rng::with_seed(seed);
    let rect = || {
        let h = BOX / 2.;
        vec![
            Vec2::new(-h.x, -h.y),
            Vec2::new(h.x, -h.y),
            h,
            Vec2::new(-h.x, h.y),
        ]
    };
    let full = signed_area(&rect());
    let mut free = rect();
    let mut enemies: Vec<Vec2> = (0..ENEMIES)
        .map(|_| random_inside(&mut rng, |p| point_in(&free, p)))
        .collect();
    let mut s = Stats::default();
    let mut fails = 0;
    while s.claims < claims {
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
            // From a corner: between the two edges.
            let prev = free[(e + free.len() - 1) % free.len()];
            inward = (inward + (a - prev).perp().normalize()).normalize_or_zero();
        }
        let f = free.clone();
        // The input rule: a point closer than EPS to the boundary ends the line there.
        let hit = |p: Vec2, q: Vec2| {
            let crossing = first_hit(&f, p, q);
            // The first step leaves the boundary: only a crossing ends it.
            if locate(&f, p).2.distance(p) < EPS {
                return crossing.map(|c| (c, c));
            }
            let limit = crossing.unwrap_or(q);
            let steps = ((limit - p).length() / (EPS * 0.5)).ceil().max(1.) as i32;
            (1..=steps)
                .map(|k| {
                    let r = p.lerp(limit, k as f32 / steps as f32);
                    (r, locate(&f, r).2)
                })
                .find(|(r, b)| b.distance(*r) < EPS)
                .or(crossing.map(|c| (c, c)))
        };
        let Some(path) = random_path(&mut rng, |p| point_in(&f, p), hit, start, inward) else {
            fails += 1;
            if fails > 100_000 {
                eprintln!(
                    "stalled at {} vertices, {:.3} claimed",
                    free.len(),
                    1. - signed_area(&free) / full
                );
                s.stalls += 1;
                fails = 0;
                free = rect();
            }
            continue;
        };
        fails = 0;
        s.path_points += path.len();
        let t0 = Instant::now();
        let [one, two] = split(&free, &path);
        let (a1, a2) = (signed_area(&one), signed_area(&two));
        let in_one = enemies.iter().filter(|e| point_in(&one, **e)).count();
        let (claimed, kept) = if in_one == 0 { (one, two) } else { (two, one) };
        let dt = t0.elapsed();
        s.time += dt;
        s.max_time = s.max_time.max(dt);
        s.times.push(dt);
        s.claims += 1;
        let ok = is_simple(&claimed)
            && is_simple(&kept)
            && a1 > 0.
            && a2 > 0.
            && ((a1 + a2) - signed_area(&free)).abs() < 1e-3 * full;
        if !ok {
            if s.invalid < 8 {
                let (i0, t0, _) = locate(&free, path[0]);
                let (i1, t1, _) = locate(&free, *path.last().unwrap());
                eprintln!(
                    "invalid: start t {t} -> edge {i0} t {t0:.4}, end edge {i1} t {t1:.4}, n {}, simple {} {}, areas {a1:.1} {a2:.1} sum err {:.3}, path {}",
                    free.len(),
                    is_simple(&claimed),
                    is_simple(&kept),
                    (a1 + a2) - signed_area(&free),
                    path.len()
                );
            }
            s.invalid += 1;
            free = rect();
            continue;
        }
        // Enemies in the claimed part are eaten (both sides had enemies).
        enemies.retain(|e| point_in(&kept, *e));
        free = kept;
        s.max_vertices = s.max_vertices.max(free.len());
        while enemies.len() < ENEMIES {
            enemies.push(random_inside(&mut rng, |p| point_in(&free, p)));
        }
        if 1. - signed_area(&free) / full >= TARGET {
            s.levels += 1;
            free = rect();
            enemies.clear();
            while enemies.len() < ENEMIES {
                enemies.push(random_inside(&mut rng, |p| point_in(&free, p)));
            }
        }
    }
    s
}

// ---------------------------------------------------------------- Model B

struct Grid {
    w: i32,
    h: i32,
    cell: f32,
    /// 0 free, 1 claimed.
    cells: Vec<u8>,
}

impl Grid {
    fn new(cell: f32) -> Self {
        let (w, h) = ((BOX.x / cell) as i32, (BOX.y / cell) as i32);
        Self {
            w,
            h,
            cell,
            cells: vec![0; (w * h) as usize],
        }
    }
    fn at(&self, p: Vec2) -> (i32, i32) {
        (
            ((p.x + BOX.x / 2.) / self.cell).floor() as i32,
            ((p.y + BOX.y / 2.) / self.cell).floor() as i32,
        )
    }
    fn free(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h && self.cells[(y * self.w + x) as usize] == 0
    }
    /// Marks the cells under a segment, with no diagonal gaps (4-connected).
    fn stamp(&self, line: &mut Vec<(i32, i32)>, p: Vec2, q: Vec2) {
        let steps = ((q - p).length() / (self.cell * 0.25)).ceil().max(1.) as i32;
        let mut last = self.at(p);
        line.push(last);
        for k in 1..=steps {
            let c = self.at(p.lerp(q, k as f32 / steps as f32));
            if c != last {
                if c.0 != last.0 && c.1 != last.1 {
                    line.push((c.0, last.1));
                }
                line.push(c);
                last = c;
            }
        }
    }
    fn claimed_fraction(&self) -> f32 {
        self.cells.iter().filter(|c| **c == 1).count() as f32 / self.cells.len() as f32
    }
}

/// Claims the line and every free region with no enemy; if all regions hold enemies, the
/// smallest one is claimed and its enemies are eaten.
fn grid_claim(g: &mut Grid, path: &[Vec2], enemies: &mut Vec<Vec2>) {
    let mut line = vec![];
    for w in path.windows(2) {
        g.stamp(&mut line, w[0], w[1]);
    }
    for (x, y) in line {
        if x >= 0 && y >= 0 && x < g.w && y < g.h {
            g.cells[(y * g.w + x) as usize] = 1;
        }
    }
    let mut label = vec![0u32; g.cells.len()];
    let mut sizes = vec![0usize];
    let mut owners = vec![false];
    let mut stack = vec![];
    for y in 0..g.h {
        for x in 0..g.w {
            let i = (y * g.w + x) as usize;
            if g.cells[i] != 0 || label[i] != 0 {
                continue;
            }
            let id = sizes.len() as u32;
            sizes.push(0);
            owners.push(false);
            label[i] = id;
            stack.push((x, y));
            while let Some((cx, cy)) = stack.pop() {
                sizes[id as usize] += 1;
                for (nx, ny) in [(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)] {
                    if g.free((nx, ny)) {
                        let j = (ny * g.w + nx) as usize;
                        if label[j] == 0 {
                            label[j] = id;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
        }
    }
    for e in enemies.iter() {
        let (x, y) = g.at(*e);
        if g.free((x, y)) {
            owners[label[(y * g.w + x) as usize] as usize] = true;
        }
    }
    let mut claim: Vec<bool> = owners.iter().map(|o| !o).collect();
    if claim.iter().skip(1).all(|c| !c) && sizes.len() > 2 {
        let smallest = (1..sizes.len()).min_by_key(|i| sizes[*i]).unwrap();
        claim[smallest] = true;
    }
    for i in 0..g.cells.len() {
        if g.cells[i] == 0 && claim[label[i] as usize] && sizes.len() > 2 {
            g.cells[i] = 1;
        }
    }
    let gg = &*g;
    enemies.retain(|e| gg.free(gg.at(*e)));
}

fn model_b(seed: u64, claims: u32, cell: f32) -> Stats {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut g = Grid::new(cell);
    let mut enemies = vec![];
    let mut s = Stats::default();
    while s.claims < claims {
        while enemies.len() < ENEMIES {
            let gg = &g;
            enemies.push(random_inside(&mut rng, |p| gg.free(gg.at(p))));
        }
        // Start on the edge of a free cell next to a wall or a claimed cell.
        let (x, y) = (rng.i32(0..g.w), rng.i32(0..g.h));
        if !g.free((x, y)) {
            continue;
        }
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let Some(&(dx, dy)) = dirs.iter().find(|(dx, dy)| !g.free((x + dx, y + dy))) else {
            continue;
        };
        let centre = Vec2::new(
            (x as f32 + 0.5) * cell - BOX.x / 2.,
            (y as f32 + 0.5) * cell - BOX.y / 2.,
        );
        let out = Vec2::new(dx as f32, dy as f32);
        let start = centre + out * cell * 0.5;
        let gg = &g;
        let inside = |p: Vec2| gg.free(gg.at(p));
        let hit = |p: Vec2, q: Vec2| {
            let steps = ((q - p).length() / (cell * 0.25)).ceil().max(1.) as i32;
            (1..=steps)
                .map(|k| p.lerp(q, k as f32 / steps as f32))
                .find(|r| !gg.free(gg.at(*r)))
                .map(|r| (r, r))
        };
        let Some(path) = random_path(&mut rng, inside, hit, start, -out) else {
            continue;
        };
        s.path_points += path.len();
        let t0 = Instant::now();
        grid_claim(&mut g, &path, &mut enemies);
        let progress = g.claimed_fraction();
        let dt = t0.elapsed();
        s.time += dt;
        s.max_time = s.max_time.max(dt);
        s.times.push(dt);
        s.claims += 1;
        if progress >= TARGET {
            s.levels += 1;
            g = Grid::new(cell);
            enemies.clear();
        }
    }
    s
}

fn report(name: &str, s: &Stats) {
    let mut t = s.times.clone();
    t.sort();
    let at = |q: f64| {
        t.get(((t.len() as f64 * q) as usize).min(t.len().saturating_sub(1)))
            .copied()
            .unwrap_or_default()
    };
    println!(
        "  p50 {:.1?} p99 {:.1?} p99.9 {:.1?} p99.99 {:.1?}",
        at(0.5),
        at(0.99),
        at(0.999),
        at(0.9999)
    );
    println!(
        "{name}: {} claims, {} levels, {} invalid, {} stalls, mean {:.1?}, max {:.1?}, mean path {} points, max free vertices {}",
        s.claims,
        s.levels,
        s.invalid,
        s.stalls,
        s.time / s.claims.max(1),
        s.max_time,
        s.path_points / s.claims.max(1) as usize,
        s.max_vertices
    );
}

fn main() {
    let claims: u32 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(2000);
    match std::env::args().nth(2).as_deref() {
        Some("B") => {
            for cell in [1., 2.] {
                report(&format!("B cell {cell}"), &model_b(1, claims, cell));
            }
        }
        Some(seed) => {
            let seed = seed.parse().unwrap();
            report(&format!("A seed {seed}"), &model_a(seed, claims));
        }
        None => panic!("usage: territory_spike <claims> <seed | B>"),
    }
}
