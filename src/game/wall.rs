use avian2d::prelude::*;
use bevy::prelude::*;

use crate::theme::palette::{BASE_GRAY, NEON_GRAY};

use super::{
    AppSystems, PausableSystems,
    skins::{self, SkinAssets},
};
use crate::{save::Save, screens::Screen};

pub const WALL_THICKNESS: f32 = 35.;
pub const WALL_THICKNESS_TOP: f32 = 65.;
/// The field height stays from 3:4 (a portrait tablet) to 9:21; outside, the camera
/// letterboxes it.
const FIELD_HEIGHTS: (f32, f32) = (600., 1050.);

pub(super) fn plugin(app: &mut App) {
    app.add_message::<WallEvent>()
        .init_resource::<Field>()
        .add_systems(OnEnter(Screen::Gameplay), (fit_field, spawn_walls).chain())
        .add_systems(
            Update,
            handle_wall_events
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

/// The playfield in world units, walls included ([080], D21): 450 wide, and as tall as the
/// screen shape at the start of a run, so it fills a phone. The camera fits it into the
/// window. The default is the desktop window size.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Deref)]
pub struct Field(pub Vec2);

impl Default for Field {
    fn default() -> Self {
        Self(Vec2::new(450., 800.))
    }
}

impl Field {
    /// The field for a view of `size` (any unit): the default width, the view's shape.
    pub fn for_view(size: Vec2) -> Self {
        let width = Self::default().x;
        let height = (width * size.y / size.x).round();
        Self(Vec2::new(
            width,
            height.clamp(FIELD_HEIGHTS.0, FIELD_HEIGHTS.1),
        ))
    }

    /// The corners of the playable box inside the walls.
    pub fn inner_corners(&self) -> WallCorners {
        let half = self.0 / 2.;
        let (inner_right, inner_left) = (half.x - WALL_THICKNESS, -half.x + WALL_THICKNESS);
        let inner_top = half.y - WALL_THICKNESS_TOP;
        let inner_bottom = -half.y + WALL_THICKNESS;
        WallCorners {
            top_right: Vec2::new(inner_right, inner_top),
            top_left: Vec2::new(inner_left, inner_top),
            bottom_right: Vec2::new(inner_right, inner_bottom),
            bottom_left: Vec2::new(inner_left, inner_bottom),
        }
    }
}

/// A run takes the shape of the camera's view; a resize later only changes the letterbox.
fn fit_field(camera: Single<&Camera>, mut field: ResMut<Field>) {
    if let Some(size) = camera.logical_viewport_size() {
        field.set_if_neq(Field::for_view(size));
    }
}

#[derive(Component)]
pub struct Wall;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallCorners {
    pub top_right: Vec2,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}
#[derive(Message)]
pub enum WallEvent {
    Reset,
    Bloom,
}

/// The four walls, in the chosen wall skin ([078]): a gray mesh, or a tiled sprite.
pub fn spawn_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    field: Res<Field>,
    save: Res<Save>,
    skin_assets: Res<SkinAssets>,
) {
    let (width, height) = (field.x, field.y);

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let wall_material = materials.add(BASE_GRAY);

    let walls = [
        // Top
        (
            Vec2::new(width, WALL_THICKNESS_TOP),
            Vec2::new(0.0, half_height - WALL_THICKNESS_TOP / 2.0),
        ),
        // Right
        (
            Vec2::new(WALL_THICKNESS, height),
            Vec2::new(half_width - WALL_THICKNESS / 2.0, 0.0),
        ),
        // Bottom
        (
            Vec2::new(width, WALL_THICKNESS),
            Vec2::new(0.0, -half_height + WALL_THICKNESS / 2.0),
        ),
        // Left
        (
            Vec2::new(WALL_THICKNESS, height),
            Vec2::new(-half_width + WALL_THICKNESS / 2.0, 0.0),
        ),
    ];
    for (size, at) in walls {
        let mut wall = commands.spawn((
            Wall,
            DespawnOnExit(Screen::Gameplay),
            Transform::from_xyz(at.x, at.y, 0.0),
            RigidBody::Static,
            Collider::from(Rectangle::from_size(size)),
        ));
        skins::insert_wall_look(
            &mut wall,
            save.skins.wall,
            &skin_assets,
            size,
            &mut meshes,
            &wall_material,
        );
    }
}

/// The walls glow while a line is drawn.
fn handle_wall_events(
    mut wall_events: MessageReader<WallEvent>,
    walls: Query<&MeshMaterial2d<ColorMaterial>, With<Wall>>,
    mut sprites: Query<&mut Sprite, With<Wall>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Some(event) = wall_events.read().next() {
        let bloom = matches!(event, WallEvent::Bloom);
        for material_handle in &walls {
            if let Some(mut material) = materials.get_mut(&material_handle.0) {
                material.color = if bloom { NEON_GRAY } else { BASE_GRAY };
            }
        }
        for mut sprite in &mut sprites {
            sprite.color = if bloom {
                skins::WALL_GLOW
            } else {
                skins::WALL_TINT
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_is_450_wide_and_takes_the_view_shape_within_the_clamp() {
        let field = |w, h| Field::for_view(Vec2::new(w, h)).0;
        assert_eq!(
            field(450., 800.),
            Vec2::new(450., 800.),
            "the desktop window"
        );
        assert_eq!(field(1080., 2400.), Vec2::new(450., 1000.), "a 20:9 phone");
        assert_eq!(
            field(393., 852.),
            Vec2::new(450., 976.),
            "an iPhone 16, rounded"
        );
        assert_eq!(field(820., 1180.), Vec2::new(450., 648.), "a portrait iPad");
        assert_eq!(
            field(1920., 1080.),
            Vec2::new(450., 600.),
            "landscape letterboxes"
        );
        assert_eq!(
            field(360., 1000.),
            Vec2::new(450., 1050.),
            "taller than 9:21"
        );
    }

    #[test]
    fn world_inner_corners_leave_out_the_walls() {
        assert_eq!(
            Field::default().inner_corners(),
            WallCorners {
                top_right: Vec2::new(190., 335.),
                top_left: Vec2::new(-190., 335.),
                bottom_right: Vec2::new(190., -365.),
                bottom_left: Vec2::new(-190., -365.),
            }
        );
    }
}
