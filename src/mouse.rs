use bevy::prelude::*;

use crate::{
    polygon::PathEvent,
    utils::HashableVec2,
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
    path: Vec<HashableVec2>,
    is_drawing: bool,
    last_position: Option<HashableVec2>,
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
fn would_create_intersection(path: &[HashableVec2], new_point: HashableVec2) -> bool {
    if path.len() < 3 {
        return false; // Need at least 3 existing points to form a line segment that could intersect
    }

    // Get the last point in the path
    let last_point = path[path.len() - 1];

    // The new segment will be from the last point to the new point
    let p1 = Vec2::from(last_point);
    let p2 = Vec2::from(new_point);

    // Check if this new segment intersects with any existing segment
    // Skip the last segment which shares the last point
    for i in 0..path.len() - 2 {
        let p3 = Vec2::from(path[i]);
        let p4 = Vec2::from(path[i + 1]);

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
        mouse_path.is_drawing = true;

        // Add the first point
        let point: HashableVec2 = cursor_position.into();
        mouse_path.path.push(point);
        mouse_path.last_position = Some(point);
    }

    if buttons.pressed(MouseButton::Left) && mouse_path.is_drawing && !is_in_wall {
        let current_point: HashableVec2 = cursor_position.into();

        // Only add the point if it's different from the last one
        if mouse_path.last_position.as_ref() != Some(&current_point) {
            // Check if adding this point would create a self-intersection
            if would_create_intersection(&mouse_path.path, current_point) {
                info!("Path would intersect itself! Drawing stopped.");
                mouse_path.reset();
                path_events.send(PathEvent::ClearLines);
                return;
            }

            path_events.send(PathEvent::DrawLine(vec![
                mouse_path.path[mouse_path.path.len() - 1].into(),
                current_point.into(),
            ]));
            mouse_path.path.push(current_point);
            mouse_path.last_position = Some(current_point);
        }
    }

    if buttons.just_released(MouseButton::Left) && mouse_path.is_drawing {
        info!(
            "Finished drawing path with {} points",
            mouse_path.path.len()
        );

        // Check if the path ends on a wall
        if is_in_wall {
            // Path is valid - it starts and ends on walls
            info!("Valid path created from wall to wall");
            path_events.send(PathEvent::DrawPath(
                mouse_path
                    .path
                    .clone()
                    .into_iter()
                    .map(|hv| hv.into())
                    .collect::<Vec<Vec2>>(),
            ));
        } else {
            // Path doesn't end on a wall, so it's invalid
            info!("Invalid path - doesn't end on a wall");
            path_events.send(PathEvent::ClearLines);
        }

        mouse_path.reset();
    }
}
