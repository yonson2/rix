use crate::utils::HashableVec2;
use bevy::prelude::*;

pub const LINE_THICKNESS: f32 = 3.0;
pub const CYAN: Color = Color::srgb(0.05490196, 0.45490196, 0.5647059);
pub const NEON_CYAN: Color = Color::srgb(1.054901, 1.4549019, 1.5647059);

#[derive(Event, Debug, Clone)]
pub enum PathEvent {
    DrawLine(Vec<Vec2>),
    DrawPath(Vec<Vec2>),
    ClearLines,
}

pub struct PolygonPlugin;

impl Plugin for PolygonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PathEvent>().add_systems(
            Update,
            (handle_draw_line, handle_draw_path, handle_clear_lines),
        );
    }
}

// Component to mark entities as temporary line segments
#[derive(Component)]
pub struct TempLineSegment;

// Component to mark entities as final path segments
#[derive(Component)]
pub struct PathSegment;

// Function to create a line segment between two points
fn line_segment(start: Vec2, end: Vec2, thickness: f32, color: Color) -> impl Bundle {
    let length = start.distance(end);
    let diff = start - end;
    let theta = diff.y.atan2(diff.x);
    let midpoint = (start + end) / 2.;

    let transform =
        Transform::from_xyz(midpoint.x, midpoint.y, 0.).with_rotation(Quat::from_rotation_z(theta));

    (
        Sprite {
            color,
            custom_size: Some(Vec2::new(length, thickness)),
            ..default()
        },
        transform,
    )
}

// System to handle drawing a line in real-time
fn handle_draw_line(
    mut commands: Commands,
    mut events: EventReader<PathEvent>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    for event in events.read() {
        if let PathEvent::DrawLine(points) = event {
            if points.len() < 2 {
                continue;
            }

            // Convert window coordinates to world coordinates
            let start = convert_to_world_coords(points[0], window_size);
            let end = convert_to_world_coords(points[1], window_size);

            commands.spawn((line_segment(start, end, 3.0, NEON_CYAN), TempLineSegment));
        }
    }
}

// System to handle drawing the final path
fn handle_draw_path(
    mut commands: Commands,
    mut events: EventReader<PathEvent>,
    windows: Query<&Window>,
    temp_segments: Query<Entity, With<TempLineSegment>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());

    for event in events.read() {
        if let PathEvent::DrawPath(points) = event {
            if points.len() < 2 {
                continue;
            }

            // Clear temporary line segments
            for entity in temp_segments.iter() {
                commands.entity(entity).despawn();
            }

            // Convert all points to world coordinates
            let world_points: Vec<Vec2> = points
                .iter()
                .map(|p| convert_to_world_coords(*p, window_size))
                .collect();

            // Draw final path segments
            for i in 0..world_points.len() - 1 {
                let start = world_points[i];
                let end = world_points[i + 1];

                commands.spawn((line_segment(start, end, 3.0, CYAN), PathSegment));
            }
        }
    }
}

// System to handle clearing all line segments
fn handle_clear_lines(
    mut commands: Commands,
    mut events: EventReader<PathEvent>,
    temp_segments: Query<Entity, With<TempLineSegment>>,
    path_segments: Query<Entity, With<PathSegment>>,
) {
    for event in events.read() {
        if let PathEvent::ClearLines = event {
            // Clear temporary line segments
            for entity in temp_segments.iter() {
                commands.entity(entity).despawn();
            }

            // // Clear path segments
            // for entity in path_segments.iter() {
            //     commands.entity(entity).despawn();
            // }
        }
    }
}

// Helper function to convert window coordinates to world coordinates
fn convert_to_world_coords(window_pos: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        window_pos.x - window_size.x / 2.,
        window_size.y / 2. - window_pos.y,
    )
}
