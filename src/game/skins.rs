//! Skins ([078], D24): looks for the line and the walls, unlocked by the best level the
//! player reaches. Look only: the line keeps its colliders and the walls their bodies, so a
//! seed plays the same with any skin ([067]).

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use serde::{Deserialize, Serialize};

use super::{items::star::Star, level::DespawnAfter, setup::Run, state::Phase, wall::Field};
use crate::{
    save::Save,
    theme::{FontAssets, palette::NEON_YELLOW},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SkinAssets>()
        .add_systems(OnEnter(Phase::Playing), record_level)
        .add_systems(Update, hide_marks_in_star);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineSkin {
    #[default]
    Neon,
    Laser,
    Chain,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallSkin {
    #[default]
    Plain,
    Brick,
    Circuit,
}

/// The chosen skins, in the save.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Skins {
    pub line: LineSkin,
    pub wall: WallSkin,
}

impl LineSkin {
    pub const ALL: [Self; 3] = [Self::Neon, Self::Laser, Self::Chain];

    pub fn name(self) -> &'static str {
        match self {
            Self::Neon => "NEON",
            Self::Laser => "LASER",
            Self::Chain => "CHAIN",
        }
    }

    /// The best level that unlocks it (D24).
    pub fn level(self) -> usize {
        match self {
            Self::Neon => 0,
            Self::Laser => 5,
            Self::Chain => 15,
        }
    }

    /// The line's own color and the one it pulses to.
    pub fn core(self) -> (Color, Color) {
        use crate::theme::palette::{NEON_CYAN, SUPER_NEON_CYAN};
        match self {
            Self::Neon => (NEON_CYAN, SUPER_NEON_CYAN),
            Self::Laser => (Color::srgb(0.8, 0.1, 0.15), Color::srgb(1.6, 0.25, 0.3)),
            Self::Chain => (Color::NONE, Color::NONE),
        }
    }

    fn spacing(self) -> Option<f32> {
        match self {
            Self::Neon => None,
            Self::Laser => Some(LASER_SPACING),
            Self::Chain => Some(CHAIN_SPACING),
        }
    }
}

impl WallSkin {
    pub const ALL: [Self; 3] = [Self::Plain, Self::Brick, Self::Circuit];

    pub fn name(self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::Brick => "BRICK",
            Self::Circuit => "CIRCUIT",
        }
    }

    pub fn level(self) -> usize {
        match self {
            Self::Plain => 0,
            Self::Brick => 10,
            Self::Circuit => 20,
        }
    }
}

/// The skin after `current` in `all` that `open` allows, or `current`.
pub fn next<T: Copy + PartialEq>(all: &[T], current: T, open: impl Fn(T) -> bool) -> T {
    let at = all.iter().position(|&s| s == current).unwrap_or(0);
    (1..=all.len())
        .map(|k| all[(at + k) % all.len()])
        .find(|&s| open(s))
        .unwrap_or(current)
}

/// The skins that reaching level `new` unlocks, when the best level was `old`.
pub fn unlocked_between(old: usize, new: usize) -> Vec<String> {
    let line = LineSkin::ALL
        .into_iter()
        .filter(|s| old < s.level() && s.level() <= new)
        .map(|s| format!("{} LINE", s.name()));
    let wall = WallSkin::ALL
        .into_iter()
        .filter(|s| old < s.level() && s.level() <= new)
        .map(|s| format!("{} WALLS", s.name()));
    line.chain(wall).collect()
}

const NOTICE_SECS: f32 = 3.5;
/// Above the bottom wall, clear of the tips under the top wall ([070]).
const NOTICE_Y: f32 = 150.;

/// A level start sets the best level; a skin it unlocks gets a short notice.
fn record_level(
    mut commands: Commands,
    run: Res<Run>,
    mut save: ResMut<Save>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
) {
    if run.level <= save.best_level {
        return;
    }
    let unlocked = unlocked_between(save.best_level, run.level);
    save.best_level = run.level;
    if unlocked.is_empty() {
        return;
    }
    let y = field.inner_corners().bottom_left.y + NOTICE_Y;
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        DespawnAfter::from_seconds(NOTICE_SECS),
        Text2d::new(format!(
            "NEW SKIN: {}\nPICK IT IN SKINS",
            unlocked.join(", ")
        )),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(24.),
            ..default()
        },
        TextLayout::justify(Justify::Center),
        TextColor(NEON_YELLOW),
        Transform::from_xyz(0., y, 10.),
    ));
}

/// A dash or a link along a line skin; a child of its line segment.
#[derive(Component)]
pub struct LineMark;

const LASER_SPACING: f32 = 13.;
const DASH: Vec2 = Vec2::new(7., 6.);
const DASH_COLOR: Color = Color::srgb(3., 0.35, 0.4);
const CHAIN_SPACING: f32 = 9.;
const LINK: Vec2 = Vec2::new(12., 3.);
const STEEL: Color = Color::srgb(1.05, 1.1, 1.2);

/// Where the next mark of the line goes, and how many came before.
#[derive(Default)]
pub struct MarkCursor {
    next: f32,
    count: usize,
}

/// The distances from a segment's start of the marks on it. The cursor carries on, so
/// the marks keep their spacing from one segment to the next.
fn mark_offsets(length: f32, spacing: f32, cursor: &mut MarkCursor) -> Vec<f32> {
    let mut out = vec![];
    while cursor.next <= length {
        out.push(cursor.next);
        cursor.next += spacing;
    }
    cursor.next -= length;
    out
}

/// Gives a line segment from `start` to `end` the marks of `skin`. The segment's local x
/// axis runs from `end` to `start`.
pub fn add_marks(
    segment: &mut EntityCommands,
    skin: LineSkin,
    assets: &SkinAssets,
    (start, end): (Vec2, Vec2),
    cursor: &mut MarkCursor,
) {
    let Some(spacing) = skin.spacing() else {
        return;
    };
    let length = start.distance(end);
    let offsets = mark_offsets(length, spacing, cursor);
    segment.with_children(|parent| {
        for d in offsets {
            let at = Transform::from_xyz(length / 2. - d, 0., 0.1);
            let ring = cursor.count.is_multiple_of(2);
            cursor.count += 1;
            match (skin, ring) {
                (LineSkin::Laser, _) => {
                    parent.spawn((LineMark, Sprite::from_color(DASH_COLOR, DASH), at))
                }
                (_, true) => parent.spawn((
                    LineMark,
                    Mesh2d(assets.ring.clone()),
                    MeshMaterial2d(assets.steel.clone()),
                    at.with_scale(Vec3::new(1.5, 1., 1.)),
                )),
                (_, false) => parent.spawn((LineMark, Sprite::from_color(STEEL, LINK), at)),
            };
        }
    });
}

/// The Star's rainbow line wins over a line skin while it lasts (D24).
fn hide_marks_in_star(star: Option<Res<Star>>, mut marks: Query<&mut Visibility, With<LineMark>>) {
    let shown = if star.is_some() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut marks {
        visibility.set_if_neq(shown);
    }
}

/// A wall skin's tint, at rest and while a line is drawn (the walls glow then).
pub const WALL_TINT: Color = Color::srgb(0.75, 0.75, 0.75);
pub const WALL_GLOW: Color = Color::srgb(1.8, 1.8, 1.8);

/// Gives `wall` the look of a wall of `size` in `skin`: the plain gray mesh (with
/// `material`, shared by the walls), or a tiled sprite.
pub fn insert_wall_look(
    wall: &mut EntityCommands,
    skin: WallSkin,
    assets: &SkinAssets,
    size: Vec2,
    meshes: &mut Assets<Mesh>,
    material: &Handle<ColorMaterial>,
) {
    let image = match skin {
        WallSkin::Plain => {
            wall.insert((
                Mesh2d(meshes.add(Rectangle::from_size(size))),
                MeshMaterial2d(material.clone()),
            ));
            return;
        }
        WallSkin::Brick => assets.brick.clone(),
        WallSkin::Circuit => assets.circuit.clone(),
    };
    wall.insert(Sprite {
        image,
        color: WALL_TINT,
        custom_size: Some(size),
        image_mode: SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.,
        },
        ..default()
    });
}

/// The skins' textures and meshes, made in code (own work, no licence to track).
#[derive(Resource)]
pub struct SkinAssets {
    brick: Handle<Image>,
    circuit: Handle<Image>,
    ring: Handle<Mesh>,
    steel: Handle<ColorMaterial>,
}

impl FromWorld for SkinAssets {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let brick = images.add(tile(&BRICK));
        let circuit = images.add(tile(&CIRCUIT));
        let ring = world
            .resource_mut::<Assets<Mesh>>()
            .add(Annulus::new(2.5, 4.5));
        let steel = world.resource_mut::<Assets<ColorMaterial>>().add(STEEL);
        Self {
            brick,
            circuit,
            ring,
            steel,
        }
    }
}

/// Neon mortar between dark bricks; two rows, the second offset by half a brick.
const BRICK: [&str; 16] = [
    "================",
    "=#######=#######",
    "=#######=#######",
    "=#######=#######",
    "=#######=#######",
    "=#######=#######",
    "=#######=#######",
    "=#######=#######",
    "================",
    "####=#######=###",
    "####=#######=###",
    "####=#######=###",
    "####=#######=###",
    "####=#######=###",
    "####=#######=###",
    "####=#######=###",
];

/// Green traces and gold pads on a dark board.
const CIRCUIT: [&str; 16] = [
    "................",
    ".o------o.......",
    ".|......|.......",
    ".|......o----o..",
    ".|...........|..",
    ".o--o........|..",
    "....|.....o--o..",
    "....|.....|.....",
    "....o-----o.....",
    "................",
    "..o----------o..",
    "..|..........|..",
    "..|...o......|..",
    "..|...|......o..",
    "..o---o.........",
    "................",
];

fn texel(c: char) -> [u8; 4] {
    match c {
        '=' => [255, 90, 190, 255],
        '#' => [70, 22, 50, 255],
        '-' | '|' => [60, 230, 130, 255],
        'o' => [255, 205, 80, 255],
        _ => [8, 38, 22, 255],
    }
}

/// A tile from a 16 x 16 pattern, each cell 2 x 2 pixels (sharp, not smoothed).
fn tile(rows: &[&str; 16]) -> Image {
    const SCALE: usize = 2;
    let side = 16 * SCALE;
    let mut data = Vec::with_capacity(side * side * 4);
    for y in 0..side {
        let row: Vec<char> = rows[y / SCALE].chars().collect();
        for x in 0..side {
            data.extend(texel(row[x / SCALE]));
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: side as u32,
            height: side as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skins_unlock_at_best_level_5_10_15_and_20() {
        assert_eq!(unlocked_between(0, 4), Vec::<String>::new());
        assert_eq!(
            unlocked_between(0, 1),
            Vec::<String>::new(),
            "defaults are free"
        );
        assert_eq!(unlocked_between(4, 5), ["LASER LINE"]);
        assert_eq!(unlocked_between(5, 5), Vec::<String>::new(), "once");
        assert_eq!(unlocked_between(9, 10), ["BRICK WALLS"]);
        assert_eq!(unlocked_between(14, 15), ["CHAIN LINE"]);
        assert_eq!(unlocked_between(19, 20), ["CIRCUIT WALLS"]);
        assert_eq!(unlocked_between(0, 20).len(), 4);
    }

    #[test]
    fn a_level_start_records_the_best_level_and_tells_of_a_new_skin() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(FontAssets {
            main: Handle::default(),
        });
        world.init_resource::<Field>();
        world.insert_resource(Save {
            best_level: 4,
            ..default()
        });
        let notices = |world: &mut World| -> Vec<String> {
            world
                .query::<&Text2d>()
                .iter(world)
                .map(|t| t.0.clone())
                .collect()
        };
        for (level, best, shown) in [(3, 4, 0), (5, 5, 1), (5, 5, 1), (6, 6, 1)] {
            let mut run = Run::with_seed(1);
            run.level = level;
            world.insert_resource(run);
            world.run_system_once(record_level).unwrap();
            assert_eq!(world.resource::<Save>().best_level, best);
            assert_eq!(notices(&mut world).len(), shown, "level {level}");
        }
        assert!(notices(&mut world)[0].contains("LASER LINE"));
    }

    #[test]
    fn a_fresh_save_has_only_the_default_skins() {
        let save = Save::default();
        assert_eq!(save.skins, Skins::default());
        let open_line: Vec<_> = LineSkin::ALL
            .into_iter()
            .filter(|s| s.level() <= save.best_level)
            .collect();
        let open_wall: Vec<_> = WallSkin::ALL
            .into_iter()
            .filter(|s| s.level() <= save.best_level)
            .collect();
        assert_eq!(
            (open_line, open_wall),
            (vec![LineSkin::Neon], vec![WallSkin::Plain])
        );
    }

    #[test]
    fn next_steps_through_the_open_skins_only() {
        let open = |best: usize| move |s: LineSkin| s.level() <= best;
        assert_eq!(
            next(&LineSkin::ALL, LineSkin::Neon, open(0)),
            LineSkin::Neon
        );
        assert_eq!(
            next(&LineSkin::ALL, LineSkin::Neon, open(5)),
            LineSkin::Laser
        );
        assert_eq!(
            next(&LineSkin::ALL, LineSkin::Laser, open(5)),
            LineSkin::Neon
        );
        assert_eq!(
            next(&LineSkin::ALL, LineSkin::Laser, open(15)),
            LineSkin::Chain
        );
    }

    #[test]
    fn marks_keep_their_spacing_across_segments() {
        let mut cursor = MarkCursor::default();
        assert_eq!(mark_offsets(10., 14., &mut cursor), [0.]);
        assert_eq!(mark_offsets(20., 14., &mut cursor), [4., 18.]);
        assert_eq!(mark_offsets(5., 14., &mut cursor), Vec::<f32>::new());
        assert_eq!(
            mark_offsets(10., 14., &mut cursor),
            [7.],
            "0, 14, 28, 42 along"
        );
    }

    #[test]
    fn a_tile_is_32_pixels_from_its_pattern() {
        assert!(BRICK.iter().chain(&CIRCUIT).all(|row| row.len() == 16));
        let image = tile(&BRICK);
        assert_eq!(image.size(), UVec2::new(32, 32));
        let data = image.data.as_ref().unwrap();
        assert_eq!(data[..4], texel('='), "mortar at the top left");
        let row = 2 * 32 * 4;
        assert_eq!(
            data[row + 4 * 2..row + 4 * 3],
            texel('#'),
            "a brick below it"
        );
    }
}
