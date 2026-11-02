use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

use crate::{
    shape::{self, Shape},
    theme::palette::{CYAN, NEON_CYAN, SUPER_NEON_CYAN},
    utils::{ColorTransition, update_sprite_color},
};

use super::{
    AppSystems, PausableSystems,
    enemies::Enemy,
    events::{self, DrawPolygon, ShapeMeta},
    get_player_layer,
    state::Phase,
    wall::{self, WallCorners, WallSide},
};
use crate::screens::Screen;

const COLOR_TRANSITION_DURATION: f32 = 0.5;
const LINE_THICKNESS: f32 = 4.0;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(shape::plugin);
    app.add_systems(
        Update,
        (
            handle_draw_line,
            handle_clear_lines,
            handle_build_polygon,
            handle_draw_polygon,
            update_sprite_color,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Clone, Debug)]
pub struct PolygonData {
    pub start_wall: WallSide,
    pub end_wall: WallSide,
    pub path: Vec<Vec2>,
    pub start_polygon: Option<(Vec<Vec2>, ShapeMeta)>,
    pub end_polygon: Option<(Vec<Vec2>, ShapeMeta)>,
}

// Component to mark entities as temporary line segments
#[derive(Component)]
pub struct TempLineSegment;

// Component to mark entities as final path segments
#[derive(Component)]
pub struct PathSegment;

#[derive(Component)]
pub struct DrawnShape;

/// The outline of a claimed polygon in world coordinates; the last point repeats the first.
#[derive(Component, Clone)]
pub struct Territory(pub Vec<Vec2>);

fn handle_draw_line(mut commands: Commands, mut events: MessageReader<events::DrawLine>) {
    let mut result = vec![];
    for event in events.read() {
        let collider = Collider::from(Rectangle::new(
            f32::abs(event.0[0].distance(event.0[1])),
            LINE_THICKNESS,
        ));
        result.push((
            draw_line(event),
            get_player_layer(),
            TempLineSegment,
            DespawnOnExit(Phase::Playing),
            collider,
            ColorTransition {
                timer: Timer::from_seconds(COLOR_TRANSITION_DURATION, TimerMode::Repeating),
                from: NEON_CYAN,
                to: SUPER_NEON_CYAN,
            }, // RigidBody::Static, // I think its better if the line is ephemeral
        ));
    }

    commands.spawn_batch(result);
}

fn handle_clear_lines(
    mut commands: Commands,
    mut events: MessageReader<events::ClearLines>,
    temp_segments: Query<Entity, With<TempLineSegment>>,
) {
    for _ in events.read() {
        for entity in temp_segments.iter() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

fn handle_build_polygon(
    mut commands: Commands,
    mut build_events: MessageReader<events::BuildPolygon>,
    mut draw_events: MessageWriter<events::DrawPolygon>,
    mut activate_events: MessageWriter<events::ActivateItem>,
    enemy_nodes: Query<(&Transform, Entity), With<Enemy>>,
    item_nodes: Query<(Entity, &Transform), With<super::items::Item>>,
) {
    // Get all enemy positions and entities
    let enemy_nodes: Vec<(Vec2, Entity)> = enemy_nodes
        .iter()
        .map(|(t, e)| (t.translation.truncate(), e))
        .collect();

    // Get all item positions and entities
    let item_nodes: Vec<(Entity, Vec2)> = item_nodes
        .iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    for event in build_events.read() {
        let corners = wall::get_world_inner_corners();
        let mut start_wall = event.start_wall;
        let mut end_wall = event.end_wall;

        let mut path = event.path.clone();
        if let Some(start_polygon) = &event.start_polygon {
            start_wall = start_polygon.1.start_wall;
            let start_point = event.path.first().unwrap();
            let idx = find_closest_point_index(start_point, &start_polygon.0);
            let tip_idx = find_closest_point_index(&start_polygon.1.start_point, &start_polygon.0);
            let mut new_start = vec![];

            if tip_idx < idx {
                new_start.extend_from_slice(&start_polygon.0[tip_idx..idx]);
            } else {
                for i in (idx + 1..=tip_idx).rev() {
                    new_start.push(start_polygon.0[i]);
                }
            }

            path.splice(0..0, new_start);
        }

        if let Some(end_polygon) = &event.end_polygon {
            end_wall = end_polygon.1.start_wall;
            let end_point = event.path.last().unwrap();
            let idx = find_closest_point_index(end_point, &end_polygon.0);
            let tip_idx = find_closest_point_index(&end_polygon.1.start_point, &end_polygon.0);

            if idx < tip_idx {
                path.extend_from_slice(&end_polygon.0[idx + 1..=tip_idx]);
            } else {
                for i in (tip_idx..idx).rev() {
                    path.push(end_polygon.0[i]);
                }
            }
        }

        let polygons = get_possible_polygons(corners, start_wall, end_wall, path);
        let final_poly = get_correct_polygon(&enemy_nodes, polygons);

        // Get the final polygon (with correct winding order)
        let polygon_points = &final_poly.0;

        // Activate any items inside the polygon
        for (item_entity, item_pos) in &item_nodes {
            if is_point_in_polygon(item_pos, polygon_points) {
                // Activate the item by sending the activation event
                info!("Caught item in polygon: {item_entity}");
                activate_events.write(events::ActivateItem(
                    *item_entity,
                    super::items::Item::BasicBomb,
                ));
            }
        }

        // Despawn enemies
        for e in final_poly.1 {
            commands.trigger(events::DespawnEnemy(e)); //eating an enemy
        }

        let draw_data = DrawPolygon {
            meta: ShapeMeta {
                start_wall: event.start_wall,
                start_point: final_poly.0[0],
            },
            path: final_poly.0,
        };
        draw_events.write(draw_data);
    }
}

fn handle_draw_polygon(mut commands: Commands, mut events: MessageReader<events::DrawPolygon>) {
    for event in events.read() {
        if event.path.len() < 3 {
            warn!(
                "Received DrawPolygon event with less than 3 vertices ({}). Cannot create solid collider.",
                event.path.len()
            );
            continue;
        }

        let shape = Shape::new(shape::polygon(&event.path)).fill(CYAN);

        let Some(collider) = territory_collider(&event.path) else {
            error!(
                "Cannot build a collider for a polygon of {} vertices.",
                event.path.len()
            );
            continue;
        };

        // Scoped to the screen, not the level: the level transition fades the shapes out.
        commands.spawn((
            DrawnShape,
            DespawnOnExit(Screen::Gameplay),
            Territory(event.path.clone()),
            RigidBody::Static,
            collider,
            shape,
            event.meta,
        ));
    }
}

/// A solid collider of the polygon's fill triangles. VHACD took 4 ms or more per claim
/// ([024]).
fn territory_collider(path: &[Vec2]) -> Option<Collider> {
    let mut buffers: VertexBuffers<Vec2, u32> = VertexBuffers::new();
    FillTessellator::new()
        .tessellate_path(
            &shape::polygon(path),
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| {
                Vec2::from(v.position().to_array())
            }),
        )
        .ok()?;
    let triangles = buffers.indices.as_chunks::<3>().0.to_vec();
    Collider::try_trimesh(buffers.vertices, triangles).ok()
}

/// Creates two possible polygons from a path that starts and ends on walls
/// 1. One created by going clockwise from the end point to the start point
/// 2. One created by going counter-clockwise from the end point to the start point
pub fn get_possible_polygons(
    corners: WallCorners,
    start_side: WallSide,
    end_side: WallSide,
    path: Vec<Vec2>,
) -> (Vec<Vec2>, Vec<Vec2>) {
    let mut left_path = path.clone();
    let mut right_path = path;

    // Complete the clockwise polygon
    complete_polygon(&mut left_path, &corners, start_side, end_side, true);
    // Complete the counter-clockwise polygon
    complete_polygon(&mut right_path, &corners, start_side, end_side, false);

    (left_path, right_path)
}

/// Completes a polygon by adding corner points between the end and start of the path
fn complete_polygon(
    path: &mut Vec<Vec2>,
    corners: &WallCorners,
    start_wall: WallSide,
    end_wall: WallSide,
    clockwise: bool,
) {
    let mut current_wall = end_wall;
    let mut loop_needed = start_wall == end_wall && !clockwise;
    loop {
        if current_wall == start_wall {
            if loop_needed {
                let current_corner = current_wall.next_corner(clockwise, corners);
                path.push(current_corner);
                current_wall = current_wall.next_wall(clockwise);
                loop_needed = false;
                continue;
            } else {
                path.push(path[0]);
                return;
            }
        }
        let current_corner = current_wall.next_corner(clockwise, corners);
        path.push(current_corner);
        current_wall = current_wall.next_wall(clockwise);
    }
}

/// Determines which polygon should be drawn based on the enemies position
/// Returns the polygon where the enemy is NOT located if possible.
/// If not possible, returns the polygon and the entity that is going to be engulfed.
fn get_correct_polygon(
    enemy_points: &[(Vec2, Entity)],
    polygons: (Vec<Vec2>, Vec<Vec2>),
) -> (Vec<Vec2>, Vec<Entity>) {
    let (poly1, poly2) = polygons;

    if enemy_points.is_empty() {
        return (poly1, vec![]);
    }

    let enemies_in_poly1: Vec<(Vec2, Entity)> = enemy_points
        .iter()
        .filter(|(point, _)| is_point_in_polygon(point, &poly1))
        .cloned()
        .collect();

    let enemies_in_poly2: Vec<(Vec2, Entity)> = enemy_points
        .iter()
        .filter(|(point, _)| is_point_in_polygon(point, &poly2))
        .cloned()
        .collect();

    if enemies_in_poly1.is_empty() {
        (poly1, vec![])
    } else if enemies_in_poly2.is_empty() {
        (poly2, vec![])
    } else {
        (
            poly1,
            enemies_in_poly1.iter().map(|(_, entity)| *entity).collect(),
        )
    }
}

/// Checks if a point is inside a polygon using the ray casting algorithm
pub fn is_point_in_polygon(point: &Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false; // Not a valid polygon
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        // Check if the ray from point to the right intersects with the edge
        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < pi.x + (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y))
        {
            inside = !inside;
        }

        j = i;
    }

    inside
}

fn find_closest_point_index(point: &Vec2, polygon: &[Vec2]) -> usize {
    if polygon.is_empty() {
        return 0; // Safety return, shouldn't happen with valid polygons
    }

    let mut closest_idx = 0;
    let mut closest_distance = point.distance_squared(polygon[0]);

    for (i, poly_point) in polygon.iter().enumerate().skip(1) {
        let distance = point.distance_squared(*poly_point);
        if distance < closest_distance {
            closest_distance = distance;
            closest_idx = i;
        }
    }

    closest_idx
}

pub fn area(points: &[Vec2]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut sum = 0.0;

    // Apply the shoelace formula
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        sum += points[i].x * points[j].y;
        sum -= points[j].x * points[i].y;
    }
    0.5 * sum.abs()
}

fn line_segment(start: Vec2, end: Vec2, thickness: f32, color: Color) -> (Sprite, Transform) {
    let length = start.distance(end);
    let diff = start - end;
    let theta = diff.y.atan2(diff.x);
    let midpoint = (start + end) / 2.;

    let transform =
        Transform::from_xyz(midpoint.x, midpoint.y, 1.).with_rotation(Quat::from_rotation_z(theta));

    (
        Sprite {
            color,
            custom_size: Some(Vec2::new(length, thickness)),
            ..default()
        },
        transform,
    )
}

fn draw_line(points: &[Vec2]) -> (Sprite, Transform) {
    if points.len() < 2 {
        return (Sprite::default(), Transform::default());
    }

    line_segment(points[0], points[1], LINE_THICKNESS, NEON_CYAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORNERS: WallCorners = WallCorners {
        top_right: Vec2::new(1., 1.),
        top_left: Vec2::new(-1., 1.),
        bottom_right: Vec2::new(1., -1.),
        bottom_left: Vec2::new(-1., -1.),
    };

    fn square() -> Vec<Vec2> {
        vec![
            CORNERS.top_left,
            CORNERS.top_right,
            CORNERS.bottom_right,
            CORNERS.bottom_left,
        ]
    }

    #[test]
    fn point_in_polygon_inside_and_outside() {
        assert!(is_point_in_polygon(&Vec2::ZERO, &square()));
        assert!(!is_point_in_polygon(&Vec2::new(2., 0.), &square()));
        assert!(!is_point_in_polygon(&Vec2::ZERO, &square()[..2]));
    }

    #[test]
    fn point_in_polygon_concave_notch_is_outside() {
        let l_shape = [
            Vec2::new(0., 0.),
            Vec2::new(2., 0.),
            Vec2::new(2., 1.),
            Vec2::new(1., 1.),
            Vec2::new(1., 2.),
            Vec2::new(0., 2.),
        ];
        assert!(is_point_in_polygon(&Vec2::new(0.5, 1.5), &l_shape));
        assert!(!is_point_in_polygon(&Vec2::new(1.5, 1.5), &l_shape));
    }

    #[test]
    fn point_in_polygon_quirk_left_and_bottom_edges_inside_right_and_top_outside() {
        assert!(is_point_in_polygon(&Vec2::new(-1., 0.), &square()));
        assert!(is_point_in_polygon(&Vec2::new(0., -1.), &square()));
        assert!(!is_point_in_polygon(&Vec2::new(1., 0.), &square()));
        assert!(!is_point_in_polygon(&Vec2::new(0., 1.), &square()));
    }

    #[test]
    fn possible_polygons_wall_to_wall_split_the_field() {
        let path = vec![Vec2::new(-1., 0.), Vec2::new(1., 0.)];
        let (clockwise, counter_clockwise) =
            get_possible_polygons(CORNERS, WallSide::Left, WallSide::Right, path);
        assert_eq!(
            clockwise,
            [
                Vec2::new(-1., 0.),
                Vec2::new(1., 0.),
                CORNERS.bottom_right,
                CORNERS.bottom_left,
                Vec2::new(-1., 0.),
            ]
        );
        assert_eq!(
            counter_clockwise,
            [
                Vec2::new(-1., 0.),
                Vec2::new(1., 0.),
                CORNERS.top_right,
                CORNERS.top_left,
                Vec2::new(-1., 0.),
            ]
        );
    }

    #[test]
    fn possible_polygons_adjacent_walls_cut_a_corner() {
        let path = vec![Vec2::new(0., 1.), Vec2::new(1., 0.)];
        let (clockwise, counter_clockwise) =
            get_possible_polygons(CORNERS, WallSide::Top, WallSide::Right, path);
        assert_eq!(clockwise.len(), 6);
        assert!(is_point_in_polygon(&Vec2::new(-0.5, -0.5), &clockwise));
        assert_eq!(
            counter_clockwise,
            [
                Vec2::new(0., 1.),
                Vec2::new(1., 0.),
                CORNERS.top_right,
                Vec2::new(0., 1.)
            ]
        );
    }

    #[test]
    fn possible_polygons_same_wall_left_to_right() {
        let bump = vec![Vec2::new(-0.5, -1.), Vec2::new(0., 0.), Vec2::new(0.5, -1.)];
        let (clockwise, counter_clockwise) =
            get_possible_polygons(CORNERS, WallSide::Bottom, WallSide::Bottom, bump.clone());
        assert_eq!(clockwise, [bump.clone(), vec![bump[0]]].concat());
        assert_eq!(
            counter_clockwise,
            [
                bump.clone(),
                vec![
                    CORNERS.bottom_right,
                    CORNERS.top_right,
                    CORNERS.top_left,
                    CORNERS.bottom_left,
                    bump[0],
                ]
            ]
            .concat()
        );
    }

    #[test]
    fn possible_polygons_quirk_same_wall_right_to_left_overlaps_but_point_test_holds() {
        let bump = vec![Vec2::new(0.5, -1.), Vec2::new(0., 0.), Vec2::new(-0.5, -1.)];
        let (_, counter_clockwise) =
            get_possible_polygons(CORNERS, WallSide::Bottom, WallSide::Bottom, bump);
        // Goes back along the bottom wall over the bump's base: the outline overlaps itself.
        assert_eq!(counter_clockwise[3], CORNERS.bottom_right);
        assert!(!is_point_in_polygon(
            &Vec2::new(0., -0.5),
            &counter_clockwise
        ));
        assert!(is_point_in_polygon(
            &Vec2::new(0.8, -0.5),
            &counter_clockwise
        ));
        assert!(is_point_in_polygon(
            &Vec2::new(-0.8, -0.5),
            &counter_clockwise
        ));
        assert!(is_point_in_polygon(&Vec2::new(0., 0.5), &counter_clockwise));
    }

    #[test]
    fn correct_polygon_avoids_enemies_or_eats_those_in_the_first() {
        let mut world = World::new();
        let (a, b) = (world.spawn_empty().id(), world.spawn_empty().id());
        let path = vec![Vec2::new(-1., 0.), Vec2::new(1., 0.)];
        let polygons = get_possible_polygons(CORNERS, WallSide::Left, WallSide::Right, path);
        let (bottom, top) = polygons.clone();
        let in_bottom = Vec2::new(0., -0.5);
        let in_top = Vec2::new(0., 0.5);

        assert_eq!(
            get_correct_polygon(&[], polygons.clone()),
            (bottom.clone(), vec![])
        );
        assert_eq!(
            get_correct_polygon(&[(in_top, a)], polygons.clone()),
            (bottom.clone(), vec![])
        );
        assert_eq!(
            get_correct_polygon(&[(in_bottom, a)], polygons.clone()),
            (top, vec![])
        );
        assert_eq!(
            get_correct_polygon(&[(in_bottom, a), (in_top, b)], polygons),
            (bottom, vec![a])
        );
    }

    #[test]
    fn closest_point_index_first_wins_ties() {
        assert_eq!(find_closest_point_index(&Vec2::ZERO, &[]), 0);
        assert_eq!(
            find_closest_point_index(&Vec2::new(0.9, -0.8), &square()),
            2
        );
        assert_eq!(find_closest_point_index(&Vec2::new(0., 1.), &square()), 0);
    }

    /// An L: the notch at (15, 15) is outside. The last point repeats the first, as in
    /// `Territory`.
    fn l_territory() -> Vec<Vec2> {
        [
            (0., 0.),
            (20., 0.),
            (20., 10.),
            (10., 10.),
            (10., 20.),
            (0., 20.),
            (0., 0.),
        ]
        .map(|(x, y)| Vec2::new(x, y))
        .to_vec()
    }

    #[test]
    fn territory_collider_is_solid_in_both_windings() {
        use avian2d::{collision::collider::contact_query::intersection_test, prelude::Rotation};
        let ball = Collider::circle(1.);
        for path in [l_territory(), l_territory().into_iter().rev().collect()] {
            let territory = territory_collider(&path).unwrap();
            let touches = |at: Vec2| {
                intersection_test(
                    &territory,
                    Vec2::ZERO,
                    Rotation::default(),
                    &ball,
                    at,
                    Rotation::default(),
                )
                .unwrap()
            };
            assert!(touches(Vec2::new(5., 5.)), "a ball deep inside touches");
            assert!(touches(Vec2::new(15., 5.)));
            assert!(touches(Vec2::new(5., 15.)));
            assert!(!touches(Vec2::new(15., 15.)), "the notch is outside");
            assert!(!touches(Vec2::new(25., 5.)));
        }
    }

    #[test]
    fn a_ball_bounces_off_a_territory_in_both_windings() {
        use avian2d::prelude::*;
        use std::time::Duration;

        for path in [l_territory(), l_territory().into_iter().rev().collect()] {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                AssetPlugin::default(),
                bevy::scene::ScenePlugin,
                bevy::mesh::MeshPlugin,
                TransformPlugin,
                PhysicsPlugins::default(),
            ))
            .insert_resource(Gravity::ZERO)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f64(1. / 60.),
            ));
            app.finish();
            app.world_mut()
                .spawn((RigidBody::Static, territory_collider(&path).unwrap()));
            let ball = app
                .world_mut()
                .spawn((
                    RigidBody::Dynamic,
                    Collider::circle(2.),
                    Transform::from_xyz(60., 5., 0.),
                    LinearVelocity(Vec2::new(-350., 0.)),
                    Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
                ))
                .id();
            for _ in 0..30 {
                app.update();
                let at = app.world().get::<Position>(ball).unwrap().0;
                assert!(!is_point_in_polygon(&at, &path), "the ball entered at {at}");
            }
            let velocity = app.world().get::<LinearVelocity>(ball).unwrap().x;
            assert!(velocity > 300., "the ball bounced back: {velocity}");
        }
    }
}
