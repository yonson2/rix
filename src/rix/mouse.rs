use bevy::prelude::*;

use crate::rix::{
    polygon::PathEvent,
    wall::{WALL_MARGIN, WALL_THICKNESS, Wall},
};

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePath>();
        app.add_systems(Update, handle_click);
        app.add_event::<PathEvent>();
    }
}

#[derive(Resource, Default, Debug)]
pub struct MousePath {
    path: Vec<Vec2>,
    is_drawing: bool,
    last_position: Option<Vec2>,
}

impl MousePath {
    fn reset(&mut self) {
        self.path.clear();
        self.is_drawing = false;
        self.last_position = None;
    }
}

// Function to check if a point is inside a wall
fn is_point_in_wall(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: usize,
    window_size: Vec2,
) -> bool {
    // Get wall position in world coordinates
    let wall_pos = wall_transform.translation();

    // Calculate wall dimensions based on side
    let (wall_width, wall_height) = match wall_side {
        0 | 2 => {
            // Top or bottom wall (horizontal)
            let width = window_size.x - 2. * WALL_MARGIN; // Playable width
            (width, WALL_THICKNESS)
        }
        _ => {
            // Left or right wall (vertical)
            let height = window_size.y - 2. * WALL_MARGIN; // Playable height
            (WALL_THICKNESS, height)
        }
    };

    let detection_margin = 3.;
    // Calculate wall bounds in world coordinates
    let half_width = wall_width / 2. + detection_margin;
    let half_height = wall_height / 2. + detection_margin;

    let wall_min_x = wall_pos.x - half_width;
    let wall_max_x = wall_pos.x + half_width;
    let wall_min_y = wall_pos.y - half_height;
    let wall_max_y = wall_pos.y + half_height;

    // Convert cursor position from window coordinates to world coordinates
    // Window coordinates: origin at top-left, y increases downward
    // World coordinates: origin at center, y increases upward
    let world_x = cursor_pos.x - window_size.x / 2.;
    let world_y = window_size.y / 2. - cursor_pos.y;

    // Check if point is within wall bounds
    world_x >= wall_min_x && world_x <= wall_max_x && world_y >= wall_min_y && world_y <= wall_max_y
}

// Function to check if adding a new point would create an intersection
fn would_create_intersection(path: &[Vec2], new_point: Vec2) -> bool {
    if path.len() < 3 {
        return false; // Need at least 3 existing points to form a line segment that could intersect
    }

    // Get the last point in the path
    let last_point = path[path.len() - 1];

    // The new segment will be from the last point to the new point
    let p1 = last_point;
    let p2 = new_point;

    // Check if this new segment intersects with any existing segment
    // Skip the last segment which shares the last point
    for i in 0..path.len() - 2 {
        let p3 = path[i];
        let p4 = path[i + 1];

        if line_segments_intersect(p1, p2, p3, p4) {
            return true;
        }
    }

    false
}

fn line_segments_intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> bool {
    // Calculate direction vectors
    let r = p2 - p1;
    let s = p4 - p3;

    // Calculate the cross product
    let cross_rs = r.x * s.y - r.y * s.x;

    // If cross product is zero, lines are parallel
    if cross_rs.abs() < f32::EPSILON {
        return false;
    }

    let q_minus_p = p3 - p1;
    let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / cross_rs;
    let u = (q_minus_p.x * r.y - q_minus_p.y * r.x) / cross_rs;

    // Check if intersection point is within both line segments
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

// Function to calculate the inner edge point of a wall
fn calculate_inner_edge_point(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: usize,
    window_size: Vec2,
) -> Vec2 {
    // Convert cursor position to world coordinates
    let world_x = cursor_pos.x - window_size.x / 2.;
    let world_y = window_size.y / 2. - cursor_pos.y;

    // Get wall position in world coordinates
    let wall_pos = wall_transform.translation();

    // Calculate the inner edge point based on wall side
    let inner_point = match wall_side {
        0 => {
            // Top wall - inner edge is at the bottom of the wall
            Vec2::new(world_x, wall_pos.y - WALL_THICKNESS / 2.0)
        }
        1 => {
            // Right wall - inner edge is at the left of the wall
            Vec2::new(wall_pos.x - WALL_THICKNESS / 2.0, world_y)
        }
        2 => {
            // Bottom wall - inner edge is at the top of the wall
            Vec2::new(world_x, wall_pos.y + WALL_THICKNESS / 2.0)
        }
        _ => {
            // Left wall - inner edge is at the right of the wall
            Vec2::new(wall_pos.x + WALL_THICKNESS / 2.0, world_y)
        }
    };

    // Convert back to window coordinates
    Vec2::new(
        inner_point.x + window_size.x / 2.,
        window_size.y / 2. - inner_point.y,
    )
}

// Function to find which wall was clicked and get its inner edge point
fn get_wall_inner_edge(
    cursor_pos: Vec2,
    walls: &Query<(&GlobalTransform, &Wall)>,
    window_size: Vec2,
) -> Option<Vec2> {
    for (transform, wall) in walls.iter() {
        if is_point_in_wall(cursor_pos, transform, wall.side, window_size) {
            return Some(calculate_inner_edge_point(
                cursor_pos,
                transform,
                wall.side,
                window_size,
            ));
        }
    }
    None
}

pub fn handle_click(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    walls: Query<(&GlobalTransform, &Wall)>,
    mut mouse_path: ResMut<MousePath>,
    mut path_events: EventWriter<PathEvent>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());

    let is_in_wall = walls
        .iter()
        .any(|(transform, w)| is_point_in_wall(cursor_position, transform, w.side, window_size));

    if buttons.just_pressed(MouseButton::Left) && is_in_wall {
        info!("Starting drawing");
        mouse_path.reset();
        path_events.send(PathEvent::ClearLines);
        mouse_path.is_drawing = true;

        // Get the inner edge point of the wall instead of using cursor position directly
        if let Some(inner_edge_point) = get_wall_inner_edge(cursor_position, &walls, window_size) {
            // Add the adjusted first point
            mouse_path.path.push(inner_edge_point);
            mouse_path.last_position = Some(inner_edge_point);
        } else {
            // Fallback to cursor position if something went wrong
            mouse_path.path.push(cursor_position);
            mouse_path.last_position = Some(cursor_position);
        }
    }

    if buttons.pressed(MouseButton::Left) && mouse_path.is_drawing {
        let current_point = cursor_position;

        // Only add the point if it's different from the last one
        if !is_approximately_equal(mouse_path.last_position.unwrap_or_default(), current_point) {
            // Check if adding this point would create a self-intersection
            if would_create_intersection(&mouse_path.path, current_point) {
                info!("Path would intersect itself! Drawing stopped.");
                mouse_path.reset();
                path_events.send(PathEvent::ClearLines);
                return;
            }

            // Check if the current point is on a wall (for completing the path)
            if is_in_wall && mouse_path.path.len() > 1 {
                // Get the inner edge point of the wall for the end point
                if let Some(inner_edge_point) =
                    get_wall_inner_edge(cursor_position, &walls, window_size)
                {
                    // Draw the final line segment to the inner edge
                    path_events.send(PathEvent::DrawLine(vec![
                        mouse_path.path[mouse_path.path.len() - 1],
                        inner_edge_point,
                    ]));

                    // Add the adjusted end point
                    mouse_path.path.push(inner_edge_point);

                    // Path is valid - it starts and ends on walls
                    info!("Valid path created from wall to wall");
                    path_events.send(PathEvent::DrawPath(mouse_path.path.clone()));
                    path_events.send(PathEvent::ClearLines);
                    mouse_path.reset();
                    return;
                }
            }

            // If not completing the path, just add the point normally
            if !is_in_wall {
                path_events.send(PathEvent::DrawLine(vec![
                    mouse_path.path[mouse_path.path.len() - 1],
                    current_point,
                ]));
                mouse_path.path.push(current_point);
                mouse_path.last_position = Some(current_point);
            }
        }
    }

    if buttons.just_released(MouseButton::Left) && mouse_path.is_drawing {
        info!("Mouse released, clearing any incomplete path");
        path_events.send(PathEvent::ClearLines);
        mouse_path.reset();
    }
}

// Helper function to compare Vec2 values with a small epsilon
fn is_approximately_equal(a: Vec2, b: Vec2) -> bool {
    const EPSILON: f32 = 0.001;
    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
}
