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

use super::{
    items::star::Star,
    level::DespawnAfter,
    setup::Run,
    state::{IsPaused, Phase},
    wall::Field,
};
use crate::{
    save::{SOUNDTRACK_LEVEL, Save},
    theme::{FontAssets, palette::NEON_YELLOW},
    utils::update_sprite_color,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SkinAssets>()
        .add_systems(OnEnter(Phase::Playing), record_level)
        .add_systems(
            Update,
            (
                hide_marks_in_star,
                // The Star's rainbow and the death's flash win over the beam.
                pulse_laser
                    .after(update_sprite_color)
                    .run_if(not(resource_exists::<Star>))
                    .run_if(not(in_state(IsPaused::Dying))),
            ),
        );
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
            Self::Laser => (LASER_RED, LASER_RED),
            Self::Chain => (Color::NONE, Color::NONE),
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

/// The notice for what a new best level unlocks: the soundtrack ([089]) and the skins.
fn unlock_notice(old: usize, new: usize) -> Option<String> {
    let mut lines = vec![];
    if old < SOUNDTRACK_LEVEL && SOUNDTRACK_LEVEL <= new {
        lines.push("SOUNDTRACK UNLOCKED\nTURN IT ON IN SETTINGS".to_string());
    }
    let skins = unlocked_between(old, new);
    if !skins.is_empty() {
        lines.push(format!("NEW SKIN: {}\nPICK IT IN SKINS", skins.join(", ")));
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

const NOTICE_SECS: f32 = 3.5;
/// Above the bottom wall, clear of the tips under the top wall ([070]).
const NOTICE_Y: f32 = 150.;

/// A level start sets the best level; what it unlocks gets a short notice. The first run
/// (the one with the tutorial) counts for nothing ([092]).
fn record_level(
    mut commands: Commands,
    run: Res<Run>,
    mut save: ResMut<Save>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
) {
    if save.stats.runs == 0 || run.level <= save.best_level {
        return;
    }
    let notice = unlock_notice(save.best_level, run.level);
    save.best_level = run.level;
    let Some(notice) = notice else {
        return;
    };
    let y = field.inner_corners().bottom_left.y + NOTICE_Y;
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        DespawnAfter::from_seconds(NOTICE_SECS),
        Text2d::new(notice),
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

/// A part of a line skin drawn over the segment (the laser's core and tip, a chain link);
/// a child of its line segment.
#[derive(Component)]
pub struct LineMark;

/// The hot beam ([093], D27): a red line that blooms, a white-hot core, light pulses that
/// run to the finger, a flicker and a glowing dot at the tip.
const LASER_RED: Color = Color::srgb(2.4, 0.2, 0.25);
const LASER_CORE: Color = Color::srgb(2.8, 2.3, 2.3);
const LASER_CORE_WIDTH: f32 = 1.5;
/// The red glow: a soft strip this wide, in place of the plain segment sprite.
const LASER_GLOW_WIDTH: f32 = 14.;
const TIP_RADIUS: f32 = 4.;
/// World units per second toward the finger, and between two pulses.
const PULSE_SPEED: f32 = 420.;
const PULSE_GAP: f32 = 90.;
/// How much brighter a pulse's crest is than the beam.
const PULSE_GAIN: f32 = 1.5;
const CHAIN_SPACING: f32 = 9.;
const LINK: Vec2 = Vec2::new(12., 3.);
const STEEL: Color = Color::srgb(1.05, 1.1, 1.2);

/// Where the next mark of the line goes, how many came before, how long the line is so
/// far, and the laser's tip.
#[derive(Default)]
pub struct MarkCursor {
    next: f32,
    count: usize,
    along: f32,
    tip: Option<Entity>,
}

/// A laser segment, and how far along the line it starts.
#[derive(Component)]
pub struct Beam {
    along: f32,
}

/// The glowing dot at the finger's end of a laser line.
#[derive(Component)]
struct LaserTip;

/// 1 at a pulse's crest, near 0 between pulses; the crests run to the finger.
fn pulse(along: f32, secs: f32) -> f32 {
    let phase = (along - secs * PULSE_SPEED) / PULSE_GAP;
    let wave = 0.5 + 0.5 * ops::cos(std::f32::consts::TAU * phase);
    ops::powf(wave, 6.)
}

fn flicker(secs: f32) -> f32 {
    1. + 0.06 * ops::sin(secs * 53.) + 0.04 * ops::sin(secs * 97.)
}

/// Draws only.
fn pulse_laser(
    time: Res<Time>,
    mut beams: Query<(&Beam, &mut Sprite)>,
    mut tips: Query<&mut Transform, With<LaserTip>>,
) {
    let secs = time.elapsed_secs();
    let flicker = flicker(secs);
    let red = LASER_RED.to_srgba();
    for (beam, mut sprite) in &mut beams {
        let glow = flicker * (1. + PULSE_GAIN * pulse(beam.along, secs));
        sprite.color = Color::srgb(red.red * glow, red.green * glow, red.blue * glow);
    }
    for mut tip in &mut tips {
        tip.scale = Vec3::splat(flicker * (1. + 0.2 * ops::sin(secs * 31.)));
    }
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
    let length = start.distance(end);
    match skin {
        LineSkin::Neon => {}
        LineSkin::Laser => {
            if let Some(old) = cursor.tip.take() {
                segment.commands().entity(old).try_despawn();
            }
            segment.insert((
                Beam {
                    along: cursor.along,
                },
                Sprite {
                    image: assets.glow.clone(),
                    color: LASER_RED,
                    custom_size: Some(Vec2::new(length, LASER_GLOW_WIDTH)),
                    ..default()
                },
            ));
            segment.with_children(|parent| {
                parent.spawn((
                    LineMark,
                    Sprite::from_color(LASER_CORE, Vec2::new(length, LASER_CORE_WIDTH)),
                    Transform::from_xyz(0., 0., 0.1),
                ));
                let tip = parent.spawn((
                    LineMark,
                    LaserTip,
                    Mesh2d(assets.dot.clone()),
                    MeshMaterial2d(assets.hot.clone()),
                    Transform::from_xyz(-length / 2., 0., 0.2),
                ));
                cursor.tip = Some(tip.id());
            });
        }
        LineSkin::Chain => {
            let offsets = mark_offsets(length, CHAIN_SPACING, cursor);
            segment.with_children(|parent| {
                for d in offsets {
                    let at = Transform::from_xyz(length / 2. - d, 0., 0.1);
                    let ring = cursor.count.is_multiple_of(2);
                    cursor.count += 1;
                    if ring {
                        parent.spawn((
                            LineMark,
                            Mesh2d(assets.ring.clone()),
                            MeshMaterial2d(assets.steel.clone()),
                            at.with_scale(Vec3::new(1.5, 1., 1.)),
                        ));
                    } else {
                        parent.spawn((LineMark, Sprite::from_color(STEEL, LINK), at));
                    }
                }
            });
        }
    }
    cursor.along += length;
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
    dot: Handle<Mesh>,
    hot: Handle<ColorMaterial>,
    glow: Handle<Image>,
}

impl FromWorld for SkinAssets {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let brick = images.add(tile(&BRICK));
        let circuit = images.add(tile(&CIRCUIT));
        let glow = images.add(glow_strip());
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let ring = meshes.add(Annulus::new(2.5, 4.5));
        let dot = meshes.add(Circle::new(TIP_RADIUS));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let steel = materials.add(STEEL);
        let hot = materials.add(LASER_CORE);
        Self {
            brick,
            circuit,
            ring,
            steel,
            dot,
            hot,
            glow,
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

/// A white strip across the line: opaque in the middle, fading to clear at both edges.
fn glow_strip() -> Image {
    const SIDE: u32 = 32;
    let data = (0..SIDE)
        .flat_map(|y| {
            let d = (y as f32 + 0.5) / SIDE as f32 * 2. - 1.;
            let alpha = ops::exp(-d * d / 0.12);
            [255, 255, 255, (alpha * 255.) as u8]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: 1,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::linear();
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
    fn the_soundtrack_unlocks_once_with_a_notice() {
        let at = SOUNDTRACK_LEVEL;
        assert_eq!(unlock_notice(at - 2, at - 1), None);
        assert_eq!(
            unlock_notice(at - 1, at).as_deref(),
            Some("SOUNDTRACK UNLOCKED\nTURN IT ON IN SETTINGS")
        );
        assert_eq!(unlock_notice(at, at), None, "once");
        assert_eq!(
            unlock_notice(4, 5).as_deref(),
            Some("NEW SKIN: LASER LINE\nPICK IT IN SKINS")
        );
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
            stats: crate::save::Stats {
                runs: 1,
                ..default()
            },
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
    fn the_first_run_unlocks_nothing() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(FontAssets {
            main: Handle::default(),
        });
        world.init_resource::<Field>();
        world.init_resource::<Save>();
        let mut run = Run::with_seed(1);
        run.level = 5;
        world.insert_resource(run);
        world.run_system_once(record_level).unwrap();
        assert_eq!(world.resource::<Save>().best_level, 0);
        assert_eq!(world.query::<&Text2d>().iter(&world).count(), 0);

        world.resource_mut::<Save>().stats.runs = 1;
        world.run_system_once(record_level).unwrap();
        assert_eq!(
            world.resource::<Save>().best_level,
            5,
            "the second run counts"
        );
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
    fn the_laser_pulses_run_to_the_finger() {
        let (along, secs, later) = (30., 0.4, 0.05);
        let moved = along + PULSE_SPEED * later;
        assert!((pulse(along, secs) - pulse(moved, secs + later)).abs() < 1e-4);
        assert!((pulse(0., 0.) - 1.).abs() < 1e-6, "a crest");
        assert!(pulse(PULSE_GAP / 2., 0.) < 1e-6, "dark between crests");
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
