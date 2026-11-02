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
    events::{self, DrawPolygon},
    get_player_layer,
    state::Phase,
    territory::{self, FreeRegion},
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

// Component to mark entities as temporary line segments
#[derive(Component)]
pub struct TempLineSegment;

// Component to mark entities as final path segments
#[derive(Component)]
pub struct PathSegment;

#[derive(Component)]
pub struct DrawnShape;

/// The outline of a claimed piece in world coordinates, counter-clockwise.
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
    mut free: ResMut<FreeRegion>,
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
        let sides = territory::split(&free, event);
        // The input rule keeps lines clean; a side with no area means a bad line.
        if sides.iter().any(|side| territory::signed_area(side) <= 0.) {
            warn!(
                "A line of {} points does not cut the free region.",
                event.len()
            );
            continue;
        }
        let ([claimed, kept], eaten) = get_correct_polygon(&enemy_nodes, sides);

        // Activate any items inside the polygon
        for (item_entity, item_pos) in &item_nodes {
            if is_point_in_polygon(item_pos, &claimed) {
                // Activate the item by sending the activation event
                info!("Caught item in polygon: {item_entity}");
                activate_events.write(events::ActivateItem(
                    *item_entity,
                    super::items::Item::BasicBomb,
                ));
            }
        }

        // Despawn enemies
        for e in eaten {
            commands.trigger(events::DespawnEnemy(e)); //eating an enemy
        }

        free.0 = kept;
        draw_events.write(DrawPolygon(claimed));
    }
}

fn handle_draw_polygon(mut commands: Commands, mut events: MessageReader<events::DrawPolygon>) {
    for event in events.read() {
        if event.len() < 3 {
            warn!(
                "Received DrawPolygon event with less than 3 vertices ({}). Cannot create solid collider.",
                event.len()
            );
            continue;
        }

        let shape = Shape::new(shape::polygon(event)).fill(CYAN);

        let Some(collider) = territory_collider(event) else {
            error!(
                "Cannot build a collider for a polygon of {} vertices.",
                event.len()
            );
            continue;
        };

        // Scoped to the screen, not the level: the level transition fades the shapes out.
        commands.spawn((
            DrawnShape,
            DespawnOnExit(Screen::Gameplay),
            Territory(event.0.clone()),
            RigidBody::Static,
            collider,
            shape,
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

/// Which side of a cut is claimed: the one with no enemy, or the first one when both have
/// enemies. Returns [claimed, kept] and the enemies the claim eats.
fn get_correct_polygon(
    enemy_points: &[(Vec2, Entity)],
    [one, two]: [Vec<Vec2>; 2],
) -> ([Vec<Vec2>; 2], Vec<Entity>) {
    let in_one: Vec<Entity> = enemy_points
        .iter()
        .filter(|(point, _)| is_point_in_polygon(point, &one))
        .map(|(_, entity)| *entity)
        .collect();

    if in_one.is_empty() {
        ([one, two], vec![])
    } else if !enemy_points
        .iter()
        .any(|(point, _)| is_point_in_polygon(point, &two))
    {
        ([two, one], vec![])
    } else {
        ([one, two], in_one)
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

    fn square() -> Vec<Vec2> {
        vec![
            Vec2::new(-1., 1.),
            Vec2::new(1., 1.),
            Vec2::new(1., -1.),
            Vec2::new(-1., -1.),
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
    fn correct_polygon_avoids_enemies_or_eats_those_in_the_first() {
        let mut world = World::new();
        let (a, b) = (world.spawn_empty().id(), world.spawn_empty().id());
        let bottom = vec![
            Vec2::new(-1., 0.),
            Vec2::new(-1., -1.),
            Vec2::new(1., -1.),
            Vec2::new(1., 0.),
        ];
        let top = vec![
            Vec2::new(1., 0.),
            Vec2::new(1., 1.),
            Vec2::new(-1., 1.),
            Vec2::new(-1., 0.),
        ];
        let sides = || [bottom.clone(), top.clone()];
        let in_bottom = Vec2::new(0., -0.5);
        let in_top = Vec2::new(0., 0.5);

        assert_eq!(get_correct_polygon(&[], sides()), (sides(), vec![]));
        assert_eq!(
            get_correct_polygon(&[(in_top, a)], sides()),
            (sides(), vec![])
        );
        assert_eq!(
            get_correct_polygon(&[(in_bottom, a)], sides()),
            ([top.clone(), bottom.clone()], vec![])
        );
        assert_eq!(
            get_correct_polygon(&[(in_bottom, a), (in_top, b)], sides()),
            (sides(), vec![a])
        );
    }

    /// An L: the notch at (15, 15) is outside.
    fn l_territory() -> Vec<Vec2> {
        [
            (0., 0.),
            (20., 0.),
            (20., 10.),
            (10., 10.),
            (10., 20.),
            (0., 20.),
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
