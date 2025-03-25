use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use super::{
    chain::ChainNode,
    snake,
    wall::{self, WallCorners, WallSide},
};

pub const LINE_THICKNESS: f32 = 3.0;
pub const CYAN: Color = Color::srgb(0.05490196, 0.45490196, 0.5647059);
pub const NEON_CYAN: Color = Color::srgb(1.054901, 1.4549019, 1.5647059);

#[derive(Clone, Debug)]
pub struct PolygonData {
    pub start_wall: WallSide,
    pub end_wall: WallSide,
    pub path: Vec<Vec2>,
}

pub struct PolygonPlugin;

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::DrawLine>()
            .add_event::<events::BuildPolygon>()
            .add_event::<events::ClearLines>()
            .add_event::<events::DrawPolygon>()
            .add_plugins(ShapePlugin)
            .add_systems(Update, handle_draw_line)
            .add_systems(Update, handle_clear_lines)
            .add_systems(Update, handle_build_polygon)
            .add_systems(Update, handle_draw_polygon);
    }
}

// Component to mark entities as temporary line segments
#[derive(Component)]
pub struct TempLineSegment;

// Component to mark entities as final path segments
#[derive(Component)]
pub struct PathSegment;

// Function to create a line segment between two points
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

fn handle_draw_line(
    mut commands: Commands,
    mut events: EventReader<events::DrawLine>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let window_size = Vec2::new(window.width(), window.height());

    let mut result = vec![];
    for event in events.read() {
        result.push((draw_line(event, window_size), TempLineSegment));
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
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn handle_build_polygon(
    mut build_events: EventReader<events::BuildPolygon>,
    mut draw_events: EventWriter<events::DrawPolygon>,
    window: Single<&Window, With<PrimaryWindow>>,
    snake_nodes: Query<&Transform, With<ChainNode>>,
    camera_q: Single<(&Camera, &GlobalTransform)>,
) {
    let snake_nodes: Vec<&Transform> = snake_nodes.iter().collect();
    let camera_q = camera_q.into_inner();
    for event in build_events.read() {
        let corners = wall::get_window_inner_corners(window.clone());
        let snake_points = snake::get_points(&snake_nodes, camera_q);
        let polygons = get_possible_polygons(
            corners,
            event.start_wall,
            event.end_wall,
            event.path.clone(),
        );
        let poly = get_correct_polygon(snake_points, polygons);
        draw_events.send(events::DrawPolygon(poly));
    }
}

fn handle_draw_polygon(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut events: EventReader<events::DrawPolygon>,
) {
    let window_size = Vec2::new(window.width(), window.height());
    for event in events.read() {
        if event.len() < 2 {
            return;
        }
        // Convert all points to world coordinates
        let world_points: Vec<Vec2> = event
            .iter()
            .map(|p| convert_to_world_coords(*p, window_size))
            .collect();

        let shape = shapes::Polygon {
            points: world_points,
            closed: true,
        };
        commands.spawn(ShapeBuilder::with(&shape).fill(CYAN).build());
    }
}

fn draw_line(points: &[Vec2], window_size: Vec2) -> (Sprite, Transform) {
    if points.len() < 2 {
        return (Sprite::default(), Transform::default());
    }

    // Convert window coordinates to world coordinates
    let start = convert_to_world_coords(points[0], window_size);
    let end = convert_to_world_coords(points[1], window_size);

    line_segment(start, end, LINE_THICKNESS, NEON_CYAN)
}

// Helper function to convert window coordinates to world coordinates
fn convert_to_world_coords(window_pos: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        window_pos.x - window_size.x / 2.,
        window_size.y / 2. - window_pos.y,
    )
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

/// Determines which polygon should be drawn based on the snake's position
/// Returns the polygon where the snake is NOT located
fn get_correct_polygon(snake_points: Vec<Vec2>, polygons: (Vec<Vec2>, Vec<Vec2>)) -> Vec<Vec2> {
    let (poly1, poly2) = polygons;

    // If we have no snake points, return the first polygon as a fallback
    if snake_points.is_empty() {
        return poly1;
    }

    // Check if the first snake point is in poly1
    let snake_head_in_poly1 = is_point_in_polygon(snake_points[0], &poly1);

    // Return the polygon that doesn't contain the snake head
    if snake_head_in_poly1 { poly2 } else { poly1 }
}

/// Checks if a point is inside a polygon using the ray casting algorithm
fn is_point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
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

pub mod events {
    use super::PolygonData;
    use bevy::prelude::*;

    #[derive(Event, Debug, Clone, Deref, DerefMut)]
    pub struct DrawLine(pub Vec<Vec2>);

    #[derive(Event, Debug, Clone, Deref, DerefMut)]
    pub struct BuildPolygon(pub PolygonData);

    #[derive(Event, Debug, Clone, Deref, DerefMut)]
    pub struct DrawPolygon(pub Vec<Vec2>);

    #[derive(Event, Debug, Clone)]
    pub struct ClearLines;
}
