use avian2d::prelude::{Collider, RigidBody, VhacdParameters};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;
use tess::path::Event;

use crate::{
    colors::{CYAN, NEON_CYAN, SUPER_NEON_CYAN},
    utils::{ColorTransition, update_sprite_color},
};

use super::{
    RixComponent, despawn_screen,
    enemies::Enemy,
    events::{self, DrawPolygon, ShapeMeta},
    get_player_layer,
    state::{AppState, IsPaused},
    wall::{self, WallCorners, WallSide},
};

const COLOR_TRANSITION_DURATION: f32 = 0.5;
const LINE_THICKNESS: f32 = 4.0;

pub struct PolygonPlugin;

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin);
        app.add_systems(
            Update,
            (
                handle_draw_line,
                handle_clear_lines,
                handle_build_polygon,
                handle_draw_polygon,
                update_sprite_color,
            )
                .run_if(in_state(IsPaused::Running)),
        );
        app.add_systems(OnEnter(AppState::GameOver), despawn_screen::<DrawnShape>);
        app.add_systems(OnEnter(AppState::Menu), despawn_screen::<DrawnShape>);
    }
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

fn handle_draw_line(mut commands: Commands, mut events: EventReader<events::DrawLine>) {
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
            RixComponent,
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
    mut events: EventReader<events::ClearLines>,
    temp_segments: Query<Entity, With<TempLineSegment>>,
) {
    for _ in events.read() {
        for entity in temp_segments.iter() {
            if let Some(ec) = commands.get_entity(entity) {
                ec.despawn_recursive();
            }
        }
    }
}

fn handle_build_polygon(
    mut commands: Commands,
    mut build_events: EventReader<events::BuildPolygon>,
    mut draw_events: EventWriter<events::DrawPolygon>,
    window: Single<&Window, With<PrimaryWindow>>,
    enemy_nodes: Query<(&Transform, Entity), With<Enemy>>,
) {
    let enemy_nodes: Vec<(Vec2, Entity)> = enemy_nodes
        .iter()
        .map(|(t, e)| (t.translation.truncate(), e))
        .collect();

    for event in build_events.read() {
        let corners = wall::get_world_inner_corners(&window);
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
        draw_events.send(draw_data);
    }
}

fn handle_draw_polygon(mut commands: Commands, mut events: EventReader<events::DrawPolygon>) {
    for event in events.read() {
        if event.path.len() < 3 {
            warn!(
                "Received DrawPolygon event with less than 3 vertices ({}). Cannot create solid collider.",
                event.path.len()
            );
            continue;
        }

        let polygon = shapes::Polygon {
            points: event.path.clone(),
            closed: true,
        };
        let shape = ShapeBuilder::with(&polygon).fill(CYAN).build();

        let vertices = event.path.clone();
        let num_vertices = vertices.len();

        let indices: Vec<[u32; 2]> = (0..num_vertices)
            .map(|i| [i as u32, ((i + 1) % num_vertices) as u32])
            .collect();

        let vhacd_params = VhacdParameters {
            concavity: 0.005,
            resolution: 256,
            convex_hull_approximation: true,
            ..Default::default()
        };

        // NOTE: we don't add `RixComponent` here because we do a custom fade
        //handled in level transition state.
        let collider = Collider::convex_decomposition_with_config(vertices, indices, &vhacd_params);

        commands.spawn((DrawnShape, RigidBody::Static, collider, shape, event.meta));
    }
}

/// Extract points from a Shape's path
pub fn extract_points_from_shape(shape: &Shape) -> Vec<Vec2> {
    let tolerance = 0.1;
    let mut points = Vec::new();
    let mut first_point = None;

    shape
        .path
        .iter_with_attributes()
        .for_each_flattened(tolerance, &mut |&evt| match evt {
            Event::Begin { at: (point, _) } => {
                points.push(Vec2::new(point.x, point.y));
                first_point = Some(point);
            }
            Event::Line { to: (point, _), .. } => {
                points.push(Vec2::new(point.x, point.y));
            }
            Event::End { close, .. } => {
                // If the path is closed and we have a first point, add it again
                if close && first_point.is_some() {
                    let p = first_point.unwrap();
                    points.push(Vec2::new(p.x, p.y));
                }
            }
            _ => {}
        });

    points
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
