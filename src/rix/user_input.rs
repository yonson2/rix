use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use crate::rix::{
    polygon::{PolygonData, is_point_in_polygon},
    wall::{WALL_THICKNESS, Wall},
};

use super::{
    events::{self, ShapeMeta},
    polygon::extract_points_from_shape,
    state::{AppState, IsPaused},
    wall::{WallEvent, WallSide},
};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserPath>();
        app.add_systems(Update, handle_input.run_if(in_state(IsPaused::Running)));
        app.add_systems(OnExit(AppState::InGame), cleanup_resources);
    }
}

#[derive(Resource, Default, Debug)]
pub struct UserPath {
    path: Vec<Vec2>,
    is_drawing: bool,
    last_position: Option<Vec2>,
    start: Option<WallSide>,
    end: Option<WallSide>,
}

/// `UserInput` is a common handler for abstracting the same behaviour across
/// mouse input and touch input, if on a system where both options are available
/// right now I'm choosing to support touch first and then fallback to mouse.
pub struct UserInput<'w> {
    clicks: &'w ButtonInput<MouseButton>,
    touches: &'w Touches,
    cursor_pos: Option<Vec2>,
}

#[derive(SystemParam)]
pub struct HandleInputParams<'w, 's> {
    window: Single<'w, &'static Window, With<PrimaryWindow>>,
    clicks: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    walls: Query<'w, 's, (&'static GlobalTransform, &'static Wall)>,
    user_path: ResMut<'w, UserPath>,
    cl_events: EventWriter<'w, events::ClearLines>,
    b_events: EventWriter<'w, events::BuildPolygon>,
    dl_events: EventWriter<'w, events::DrawLine>,
    wall_events: EventWriter<'w, WallEvent>,
    // Query for all polygon shapes to check for filled areas
    shapes: Query<'w, 's, (&'static Shape, &'static ShapeMeta)>,
}

impl UserPath {
    fn reset(&mut self) {
        self.path.clear();
        self.is_drawing = false;
        self.last_position = None;
    }
}

impl<'w> UserInput<'w> {
    fn new(
        clicks: &'w ButtonInput<MouseButton>,
        touches: &'w Touches,
        cursor_pos: Option<Vec2>,
    ) -> Self {
        Self {
            clicks,
            touches,
            cursor_pos,
        }
    }

    fn len(&self) -> usize {
        self.clicks.get_pressed().count() + self.touches.iter().count()
    }

    fn just_pressed(&self) -> bool {
        let touch = self.touches.any_just_pressed();
        if touch {
            touch
        } else {
            self.clicks.just_pressed(MouseButton::Left)
        }
    }

    fn pressed(&self) -> bool {
        let touch = self.touches.iter().next().is_some();
        if touch {
            touch
        } else {
            self.clicks.pressed(MouseButton::Left)
        }
    }

    fn just_released(&self) -> bool {
        let touch = self.touches.any_just_released();
        if touch {
            touch
        } else {
            self.clicks.just_released(MouseButton::Left)
        }
    }

    /// get the touch position, fallsback to cursor position depending on the system
    fn position(&self) -> Option<Vec2> {
        match self.touches.iter().next() {
            Some(t) => Some(t.position()),
            None => self.cursor_pos,
        }
    }
}

pub fn handle_input(mut p: HandleInputParams) {
    let input = UserInput::new(
        p.clicks.into_inner(),
        p.touches.into_inner(),
        p.window.cursor_position(),
    );
    if input.len() > 1 {
        info!("Can't have more than one input at the same time.");
        p.user_path.reset();
        p.cl_events.send(events::ClearLines);
    }
    let Some(user_position) = input.position() else {
        return;
    };

    let window_size = Vec2::new(p.window.width(), p.window.height());
    let user_position = convert_to_world_coords(user_position, window_size);
    let walls: Vec<(&GlobalTransform, &Wall)> = p.walls.iter().collect();
    let shapes_points: Vec<(Vec<Vec2>, ShapeMeta)> = p
        .shapes
        .iter()
        .map(|(shape, meta)| (extract_points_from_shape(shape), *meta))
        .collect();

    let maybe_edge = get_edge_point(user_position, &walls, &shapes_points, window_size);

    if input.just_pressed() && maybe_edge.is_some() {
        let (side, edge) = maybe_edge.unwrap();

        //FIX: we are calling this twice without need.
        let is_polygon_edge = p.shapes.iter().any(|(shape, _)| {
            let points = extract_points_from_shape(shape);
            is_point_in_polygon(&edge, &points)
        });

        // Only check for filled areas if we're on a wall edge, not a polygon edge
        let area_already_filled = if !is_polygon_edge {
            // Get the inward direction based on the wall side
            let inward_dir = match side {
                WallSide::Top => Vec2::new(0.0, -1.0),
                WallSide::Right => Vec2::new(-1.0, 0.0),
                WallSide::Bottom => Vec2::new(0.0, 1.0),
                WallSide::Left => Vec2::new(1.0, 0.0),
            };

            // Check multiple points at different distances
            let check_distances = [1.0, 5.0, 10.0];

            check_distances.iter().any(|&distance| {
                let check_point = edge + (inward_dir * distance);

                p.shapes.iter().any(|(shape, _)| {
                    let points = extract_points_from_shape(shape);
                    is_point_in_polygon(&check_point, &points)
                })
            })
        } else {
            false // Allow drawing from polygon edges
        };

        if area_already_filled {
            info!("Cannot start drawing - area is already filled");
            return;
        }

        info!("Starting drawing");
        p.user_path.reset();
        p.cl_events.send(events::ClearLines);
        p.wall_events.send(WallEvent::Bloom);
        p.user_path.is_drawing = true;

        p.user_path.path.push(edge);
        p.user_path.last_position = Some(edge);
        p.user_path.start = Some(side);
    } else if input.pressed() && p.user_path.is_drawing {
        let last_position = p.user_path.last_position.expect("drawing without prev");

        if is_approximately_equal(last_position, user_position) {
            return;
        }

        if would_create_intersection(&p.user_path.path, user_position) {
            p.user_path.reset();
            p.wall_events.send(WallEvent::Reset);
            p.cl_events.send(events::ClearLines);
            return;
        }

        if maybe_edge.is_some() && p.user_path.path.len() == 1 {
            let (side, edge) = maybe_edge.unwrap();
            p.user_path.path[0] = edge;
            p.user_path.last_position = Some(edge);
            p.user_path.start = Some(side);
        } else if maybe_edge.is_some() && p.user_path.path.len() > 1 {
            let (side, edge) = maybe_edge.unwrap();
            p.user_path.path.push(edge);
            p.user_path.last_position = Some(edge);
            p.user_path.end = Some(side);

            let path = p.user_path.path.clone();
            // Check if start point is on a polygon
            let start_point = path.first().unwrap();
            let start_polygon = p.shapes.iter().find_map(|(shape, meta)| {
                let points = extract_points_from_shape(shape);
                if is_point_in_polygon(start_point, &points) {
                    Some((points, *meta))
                } else {
                    None
                }
            });
            // Check if end point is on a polygon
            let end_point = path.last().unwrap();
            let end_polygon = p.shapes.iter().find_map(|(shape, meta)| {
                let points = extract_points_from_shape(shape);
                if is_point_in_polygon(end_point, &points) {
                    Some((points, *meta))
                } else {
                    None
                }
            });

            let polygon_data = PolygonData {
                start_wall: p.user_path.start.unwrap(),
                end_wall: p.user_path.end.unwrap(),
                start_polygon,
                end_polygon,
                path,
            };

            p.cl_events.send(events::ClearLines);
            p.b_events.send(events::BuildPolygon(polygon_data));
            p.wall_events.send(WallEvent::Reset);
            p.user_path.reset();
            info!("Valid path created from wall to wall");
            return;
        }

        if maybe_edge.is_none() {
            p.dl_events.send(events::DrawLine(vec![
                p.user_path.path[p.user_path.path.len() - 1],
                user_position,
            ]));
            p.user_path.path.push(user_position);
            p.user_path.last_position = Some(user_position);
        }
    } else if input.just_released() {
        if p.user_path.is_drawing {
            p.cl_events.send(events::ClearLines);
            p.wall_events.send(WallEvent::Reset);
        }
        p.user_path.reset();
    }
}

fn cleanup_resources(mut user_path: ResMut<UserPath>) {
    user_path.reset();
}

fn convert_to_world_coords(window_pos: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        window_pos.x - window_size.x / 2.,
        window_size.y / 2. - window_pos.y,
    )
}

fn is_point_in_wall(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
    window_size: Vec2,
) -> Option<WallSide> {
    let wall_pos = wall_transform.translation();

    let (wall_width, wall_height) = match wall_side {
        WallSide::Top | WallSide::Bottom => {
            let width = window_size.x;
            (width, WALL_THICKNESS)
        }
        WallSide::Right | WallSide::Left => {
            let height = window_size.y;
            (WALL_THICKNESS, height)
        }
    };

    let detection_margin = 3.;
    let half_width = wall_width / 2. + detection_margin;
    let half_height = wall_height / 2. + detection_margin;

    let wall_min_x = wall_pos.x - half_width;
    let wall_max_x = wall_pos.x + half_width;
    let wall_min_y = wall_pos.y - half_height;
    let wall_max_y = wall_pos.y + half_height;

    if cursor_pos.x >= wall_min_x
        && cursor_pos.x <= wall_max_x
        && cursor_pos.y >= wall_min_y
        && cursor_pos.y <= wall_max_y
    {
        Some(wall_side)
    } else {
        None
    }
}

fn calculate_inner_edge_point(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
) -> Vec2 {
    let wall_pos = wall_transform.translation();

    match wall_side {
        WallSide::Top => Vec2::new(cursor_pos.x, wall_pos.y - WALL_THICKNESS / 2.0),
        WallSide::Right => Vec2::new(wall_pos.x - WALL_THICKNESS / 2.0, cursor_pos.y),
        WallSide::Bottom => Vec2::new(cursor_pos.x, wall_pos.y + WALL_THICKNESS / 2.0),
        WallSide::Left => Vec2::new(wall_pos.x + WALL_THICKNESS / 2.0, cursor_pos.y),
    }
}

fn get_edge_point(
    cursor_pos: Vec2,
    walls: &[(&GlobalTransform, &Wall)],
    shapes: &[(Vec<Vec2>, ShapeMeta)],
    window_size: Vec2,
) -> Option<(WallSide, Vec2)> {
    let wall_edge = walls.iter().find_map(|(transform, wall)| {
        is_point_in_wall(cursor_pos, transform, wall.side, window_size).map(|side| {
            (
                side,
                calculate_inner_edge_point(cursor_pos, transform, wall.side),
            )
        })
    });
    if wall_edge.is_some() {
        return wall_edge;
    }
    shapes.iter().find_map(|(points, meta)| {
        if is_point_in_polygon(&cursor_pos, points) {
            Some((meta.start_wall, cursor_pos))
        } else {
            None
        }
    })
}

fn would_create_intersection(path: &[Vec2], new_point: Vec2) -> bool {
    if path.len() < 3 {
        return false;
    }

    let last_point = path[path.len() - 1];

    let p1 = last_point;
    let p2 = new_point;

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
    let r = p2 - p1;
    let s = p4 - p3;

    let cross_rs = r.x * s.y - r.y * s.x;

    if cross_rs.abs() < f32::EPSILON {
        return false;
    }

    let q_minus_p = p3 - p1;
    let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / cross_rs;
    let u = (q_minus_p.x * r.y - q_minus_p.y * r.x) / cross_rs;

    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn is_approximately_equal(a: Vec2, b: Vec2) -> bool {
    const EPSILON: f32 = 0.001;
    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
}
