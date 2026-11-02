use bevy::{
    input::{
        ButtonState,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::game::{
    AppSystems, PausableSystems, events,
    polygon::is_point_in_polygon,
    state::{IsPaused, Phase},
    territory::{FreeRegion, line_end, locate},
    wall::{FIELD, WALL_THICKNESS, WallEvent},
};

/// A press this close to the free boundary, or outside the free region, starts a line.
const START_MARGIN: f32 = WALL_THICKNESS * 0.5;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UserPath>();
    app.init_resource::<InputState>();
    app.add_systems(
        Update,
        (
            reset_input_state_flags,
            handle_input_events.before(handle_input),
            handle_input,
        )
            .chain()
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
    app.add_systems(OnExit(Phase::Playing), cleanup_resources);
    app.add_systems(OnExit(IsPaused::Paused), cancel_line);
}

#[derive(Resource, Default, Debug)]
pub struct UserPath {
    path: Vec<Vec2>,
    pub is_drawing: bool,
}

impl UserPath {
    fn reset(&mut self) {
        self.path.clear();
        self.is_drawing = false;
    }
}

#[derive(Resource, Default, Debug)]
pub struct InputState {
    is_pressed: bool,
    active_touch_id: Option<u64>,
    last_position_screen: Option<Vec2>,
    last_position_world: Option<Vec2>,
    just_pressed_this_frame: bool,
    just_released_this_frame: bool,
}

// System to reset frame-specific flags
fn reset_input_state_flags(mut input_state: ResMut<InputState>) {
    input_state.just_pressed_this_frame = false;
    input_state.just_released_this_frame = false;
}

// System to process hardware input events and update InputState
// This system translates raw events into the abstract state the main logic needs
fn handle_input_events(
    mut mouse_button_events: MessageReader<MouseButtonInput>,
    mut touch_events: MessageReader<TouchInput>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut input_state: ResMut<InputState>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = *camera;
    let to_world = |pos| to_field(camera, camera_transform, pos);

    let mut latest_cursor_pos_screen: Option<Vec2> = window.cursor_position();

    for event in cursor_moved_events.read() {
        latest_cursor_pos_screen = Some(event.position); // Overwrite with newer event positions
    }
    let latest_cursor_pos_world = latest_cursor_pos_screen.and_then(to_world);

    for event in mouse_button_events.read() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => {
                    if !input_state.is_pressed {
                        input_state.is_pressed = true;
                        input_state.just_pressed_this_frame = true;
                        input_state.active_touch_id = None; // Mouse takes over

                        input_state.last_position_screen = latest_cursor_pos_screen;
                        input_state.last_position_world = latest_cursor_pos_world;
                    } // else: Ignore if already pressed
                }
                ButtonState::Released => {
                    if input_state.is_pressed && input_state.active_touch_id.is_none() {
                        input_state.is_pressed = false;
                        input_state.just_released_this_frame = true;

                        input_state.last_position_screen = latest_cursor_pos_screen;
                        input_state.last_position_world = latest_cursor_pos_world;
                    }
                }
            }
        }
    }

    for event in touch_events.read() {
        let touch_pos_screen = event.position;
        let Some(touch_pos_world) = to_world(touch_pos_screen) else {
            continue;
        };

        match event.phase {
            TouchPhase::Started => {
                if !input_state.is_pressed {
                    input_state.is_pressed = true;
                    input_state.just_pressed_this_frame = true;
                    input_state.active_touch_id = Some(event.id);
                    input_state.last_position_screen = Some(touch_pos_screen);
                    input_state.last_position_world = Some(touch_pos_world);
                } // else: Ignore if already pressed
            }
            TouchPhase::Moved => {
                if input_state.active_touch_id == Some(event.id) {
                    input_state.last_position_screen = Some(touch_pos_screen);
                    input_state.last_position_world = Some(touch_pos_world);
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if input_state.active_touch_id == Some(event.id) {
                    input_state.is_pressed = false;
                    input_state.just_released_this_frame = true;
                    input_state.active_touch_id = None;
                    input_state.last_position_screen = Some(touch_pos_screen);
                    input_state.last_position_world = Some(touch_pos_world);
                }
            }
        }
    }

    let mouse_is_active_and_pressed =
        input_state.is_pressed && input_state.active_touch_id.is_none();

    if mouse_is_active_and_pressed {
        if input_state.last_position_world != latest_cursor_pos_world {
            input_state.last_position_screen = latest_cursor_pos_screen;
            input_state.last_position_world = latest_cursor_pos_world;
        }
    } else if !input_state.is_pressed && input_state.last_position_world != latest_cursor_pos_world
    {
        input_state.last_position_screen = latest_cursor_pos_screen;
        input_state.last_position_world = latest_cursor_pos_world;
    }
}

// Main Input Handling Logic (Original Logic, Reads from InputState)
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    mut input_state: ResMut<InputState>,
    mut user_path: ResMut<UserPath>,
    free: Res<FreeRegion>,
    mut cl_events: MessageWriter<events::ClearLines>,
    mut b_events: MessageWriter<events::BuildPolygon>,
    mut dl_events: MessageWriter<events::DrawLine>,
    mut wall_events: MessageWriter<WallEvent>,
    mut input_pos_events: MessageWriter<events::InputPosition>,
) {
    let just_pressed = input_state.just_pressed_this_frame;
    let pressed = input_state.is_pressed;
    let just_released = input_state.just_released_this_frame;

    let Some(user_position) = input_state.last_position_world else {
        if just_released && user_path.is_drawing {
            cl_events.write(events::ClearLines);
            wall_events.write(WallEvent::Reset);
            user_path.reset();
        }
        return;
    };

    if user_path.is_drawing {
        input_pos_events.write(events::InputPosition(user_position));
    }

    let maybe_start = start_point(&free, user_position);

    if just_pressed && let Some(start) = maybe_start {
        info!("Starting drawing");
        user_path.reset();
        cl_events.write(events::ClearLines);
        wall_events.write(WallEvent::Bloom);
        user_path.is_drawing = true;

        user_path.path.push(start);
    } else if pressed && user_path.is_drawing {
        let last_position = *user_path
            .path
            .last()
            .expect("Path shouldn't be empty if drawing");

        if is_approximately_equal(last_position, user_position) {
            return;
        }

        // Until the line leaves the boundary, its start follows the finger.
        if user_path.path.len() == 1
            && let Some(start) = maybe_start
        {
            user_path.path[0] = start;
            return;
        }

        let end = line_end(&free, last_position, user_position);
        let next = end.map_or(user_position, |(kept, _)| kept);
        let crosses = would_create_intersection(&user_path.path, next)
            || end.is_some_and(|(kept, snap)| {
                would_create_intersection(&[user_path.path.as_slice(), &[kept]].concat(), snap)
            });
        if crosses {
            user_path.reset();
            wall_events.write(WallEvent::Reset);
            cl_events.write(events::ClearLines);

            // Reset pressed state because the action failed/was cancelled
            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            return;
        }

        // Complete path
        if let Some((kept, snap)) = end {
            user_path.path.extend([kept, snap]);

            cl_events.write(events::ClearLines);
            b_events.write(events::BuildPolygon(user_path.path.clone()));
            wall_events.write(WallEvent::Reset);
            user_path.reset();

            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            info!("Valid path created from wall to wall");
            return;
        }

        // Draw line segment in open space
        dl_events.write(events::DrawLine(vec![last_position, user_position]));
        user_path.path.push(user_position);
    } else if just_released {
        if user_path.is_drawing {
            cl_events.write(events::ClearLines);
            wall_events.write(WallEvent::Reset);
        }
        user_path.reset(); // Reset path on release
    }
}

fn cleanup_resources(mut user_path: ResMut<UserPath>, mut input_state: ResMut<InputState>) {
    user_path.reset();
    *input_state = InputState::default(); // Reset input state
}

/// A run resumes with no line in progress: the touch that drew it ended while paused, and
/// the pause drops that message ([034]).
fn cancel_line(
    mut user_path: ResMut<UserPath>,
    mut input_state: ResMut<InputState>,
    mut cl_events: MessageWriter<events::ClearLines>,
    mut wall_events: MessageWriter<WallEvent>,
) {
    if user_path.is_drawing {
        cl_events.write(events::ClearLines);
        wall_events.write(WallEvent::Reset);
    }
    user_path.reset();
    *input_state = InputState::default();
}

/// A viewport position in the world, clamped to the field: a finger in the letterbox stays
/// on the nearest wall ([019]).
fn to_field(camera: &Camera, camera_transform: &GlobalTransform, pos: Vec2) -> Option<Vec2> {
    camera
        .viewport_to_world_2d(camera_transform, pos)
        .ok()
        .map(|p| p.clamp(-FIELD / 2., FIELD / 2.))
}

/// Where a line starts for a finger at `p`: the nearest point of the free boundary, when
/// `p` is on a wall or a claimed piece (outside the free region) or near the boundary.
fn start_point(free: &[Vec2], p: Vec2) -> Option<Vec2> {
    let edge = locate(free, p).2;
    (!is_point_in_polygon(&p, free) || edge.distance(p) < START_MARGIN).then_some(edge)
}

fn would_create_intersection(path: &[Vec2], new_point: Vec2) -> bool {
    if path.len() < 3 {
        return false;
    }

    let last_point = path[path.len() - 1];

    let p1 = last_point;
    let p2 = new_point;

    for i in 0..path.len() - 2 {
        // Check against all segments except the immediately previous one
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

    // Check if lines are parallel
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_starts_on_walls_claims_and_near_the_boundary() {
        // The box with the part below y = -300 claimed.
        let free = [
            Vec2::new(-190., -300.),
            Vec2::new(190., -300.),
            Vec2::new(190., 335.),
            Vec2::new(-190., 335.),
        ];
        let on_wall = start_point(&free, Vec2::new(210., 5.));
        assert_eq!(on_wall, Some(Vec2::new(190., 5.)));
        let on_claim = start_point(&free, Vec2::new(-50., -340.));
        assert_eq!(on_claim, Some(Vec2::new(-50., -300.)));
        let near_wall = start_point(&free, Vec2::new(175., 5.));
        assert_eq!(near_wall, Some(Vec2::new(190., 5.)));
        let in_corner = start_point(&free, Vec2::new(-220., 380.));
        assert_eq!(in_corner, Some(Vec2::new(-190., 335.)));
        assert_eq!(start_point(&free, Vec2::new(165., 5.)), None);
        assert_eq!(start_point(&free, Vec2::new(0., 0.)), None);
    }

    #[test]
    fn a_drag_from_a_claim_to_a_claim_sends_one_line_between_the_claimed_edges() {
        use crate::game::territory::{signed_area, split};
        use bevy::ecs::system::RunSystemOnce;

        let free = FreeRegion(vec![
            Vec2::new(-190., -300.),
            Vec2::new(190., -300.),
            Vec2::new(190., 335.),
            Vec2::new(-190., 335.),
        ]);
        let mut world = World::new();
        world.init_resource::<Messages<events::ClearLines>>();
        world.init_resource::<Messages<events::BuildPolygon>>();
        world.init_resource::<Messages<events::DrawLine>>();
        world.init_resource::<Messages<events::InputPosition>>();
        world.init_resource::<Messages<WallEvent>>();
        world.init_resource::<UserPath>();
        world.insert_resource(free.clone());
        let drag = [
            (-100., -340.),
            (-100., -310.),
            (-100., -250.),
            (-100., -200.),
            (100., -200.),
            (100., -250.),
            (100., -340.),
        ];
        for (k, (x, y)) in drag.into_iter().enumerate() {
            world.insert_resource(InputState {
                is_pressed: true,
                just_pressed_this_frame: k == 0,
                last_position_world: Some(Vec2::new(x, y)),
                ..default()
            });
            world.run_system_once(handle_input).unwrap();
        }

        let lines: Vec<_> = world
            .resource_mut::<Messages<events::BuildPolygon>>()
            .drain()
            .collect();
        assert_eq!(lines.len(), 1);
        let path = &lines[0].0;
        assert_eq!(
            path[0],
            Vec2::new(-100., -300.),
            "the start slid onto the claim"
        );
        assert!(path.last().unwrap().distance(Vec2::new(100., -300.)) < 1e-3);
        let [_, notch] = split(&free, path);
        assert!((signed_area(&notch) - 200. * 100.).abs() < 0.1);
        assert_eq!(world.resource::<Messages<events::DrawLine>>().len(), 4);
    }

    #[test]
    fn resume_cancels_the_line_and_releases_the_pointer() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Messages<events::ClearLines>>();
        world.init_resource::<Messages<WallEvent>>();
        world.insert_resource(UserPath {
            path: vec![Vec2::ZERO],
            is_drawing: true,
        });
        world.insert_resource(InputState {
            is_pressed: true,
            active_touch_id: Some(0),
            ..default()
        });
        world.run_system_once(cancel_line).unwrap();

        let path = world.resource::<UserPath>();
        assert!(!path.is_drawing && path.path.is_empty());
        let input = world.resource::<InputState>();
        assert!(!input.is_pressed && input.active_touch_id.is_none());
        assert_eq!(world.resource::<Messages<events::ClearLines>>().len(), 1);
        assert_eq!(world.resource::<Messages<WallEvent>>().len(), 1);
    }

    #[test]
    fn segments_intersect_inclusive_of_endpoints() {
        let (a, b) = (Vec2::new(0., 0.), Vec2::new(2., 2.));
        assert!(line_segments_intersect(
            a,
            b,
            Vec2::new(0., 2.),
            Vec2::new(2., 0.)
        ));
        assert!(line_segments_intersect(a, b, b, Vec2::new(3., 0.)));
        assert!(!line_segments_intersect(
            a,
            b,
            Vec2::new(3., 0.),
            Vec2::new(4., 0.)
        ));
        assert!(!line_segments_intersect(
            a,
            b,
            Vec2::new(0., 1.),
            Vec2::new(2., 3.)
        ));
    }

    #[test]
    fn segments_intersect_quirk_collinear_overlap_is_not_an_intersection() {
        let (a, b) = (Vec2::new(0., 0.), Vec2::new(2., 0.));
        assert!(!line_segments_intersect(
            a,
            b,
            Vec2::new(1., 0.),
            Vec2::new(3., 0.)
        ));
    }

    #[test]
    fn path_intersection_ignores_the_last_segment() {
        let path = [Vec2::new(0., 0.), Vec2::new(10., 0.), Vec2::new(10., 10.)];
        assert!(!would_create_intersection(&path[..2], Vec2::new(5., -5.)));
        assert!(would_create_intersection(&path, Vec2::new(5., -5.)));
        assert!(!would_create_intersection(&path, Vec2::new(20., 20.)));
        // Going back down through the corner touches the first segment at (10, 0).
        assert!(would_create_intersection(&path, Vec2::new(10., -5.)));
    }

    #[test]
    fn approximately_equal_below_a_thousandth() {
        assert!(is_approximately_equal(
            Vec2::ZERO,
            Vec2::new(0.0009, -0.0009)
        ));
        assert!(!is_approximately_equal(Vec2::ZERO, Vec2::new(0.002, 0.)));
    }
}
