//! Skins ([078], D24): looks for the line, the walls, the claimed area's fill ([115]) and
//! a kill ([116]), unlocked by the best level the player reaches or by games played ([114]). Look only: the line keeps its colliders
//! and the walls their bodies, so a seed plays the same with any skin ([067]).

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
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
    screens::Screen,
    theme::{
        FontAssets,
        palette::{NEON_GREEN, NEON_ORANGE, NEON_YELLOW, SUPER_NEON_CYAN},
    },
    utils::update_sprite_color,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SkinAssets>()
        .add_systems(OnEnter(Phase::Playing), record_level)
        .add_systems(
            Update,
            (
                hide_marks_in_star,
                twinkle_glints,
                animate_kill_parts,
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
    Gold,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallSkin {
    #[default]
    Plain,
    Brick,
    Circuit,
    Hazard,
}

/// The claimed area's look ([115]): the solid fill, or a pattern over it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillSkin {
    #[default]
    Solid,
    Scanlines,
    Grid,
    Stars,
}

/// A kill's look ([116]): the pixel burst, or a new one in its place.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KillSkin {
    #[default]
    Pixel,
    Shockwave,
    Firework,
}

/// The chosen skins, in the save.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Skins {
    pub line: LineSkin,
    pub wall: WallSkin,
    pub fill: FillSkin,
    pub kill: KillSkin,
}

impl LineSkin {
    pub const ALL: [Self; 4] = [Self::Neon, Self::Laser, Self::Chain, Self::Gold];

    pub fn name(self) -> &'static str {
        match self {
            Self::Neon => "NEON",
            Self::Laser => "LASER",
            Self::Chain => "CHAIN",
            Self::Gold => "GOLD",
        }
    }

    pub fn unlock(self) -> Unlock {
        match self {
            Self::Neon => Unlock::Free,
            Self::Laser => Unlock::Level(5),
            Self::Chain => Unlock::Level(15),
            Self::Gold => Unlock::Games(1000),
        }
    }

    /// The line's own color and the one it pulses to.
    pub fn core(self) -> (Color, Color) {
        use crate::theme::palette::{NEON_CYAN, SUPER_NEON_CYAN};
        match self {
            Self::Neon => (NEON_CYAN, SUPER_NEON_CYAN),
            Self::Laser => (LASER_RED, LASER_RED),
            Self::Chain => (Color::NONE, Color::NONE),
            Self::Gold => (GOLD, GOLD_PULSE),
        }
    }
}

impl WallSkin {
    pub const ALL: [Self; 4] = [Self::Plain, Self::Brick, Self::Circuit, Self::Hazard];

    pub fn name(self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::Brick => "BRICK",
            Self::Circuit => "CIRCUIT",
            Self::Hazard => "HAZARD",
        }
    }

    pub fn unlock(self) -> Unlock {
        match self {
            Self::Plain => Unlock::Free,
            Self::Brick => Unlock::Level(10),
            Self::Circuit => Unlock::Level(20),
            Self::Hazard => Unlock::Games(50),
        }
    }
}

impl FillSkin {
    pub const ALL: [Self; 4] = [Self::Solid, Self::Scanlines, Self::Grid, Self::Stars];

    pub fn name(self) -> &'static str {
        match self {
            Self::Solid => "SOLID",
            Self::Scanlines => "SCANLINES",
            Self::Grid => "GRID",
            Self::Stars => "STARS",
        }
    }

    pub fn unlock(self) -> Unlock {
        match self {
            Self::Solid => Unlock::Free,
            Self::Scanlines => Unlock::Games(5),
            Self::Grid => Unlock::Games(100),
            Self::Stars => Unlock::Games(500),
        }
    }
}

impl KillSkin {
    pub const ALL: [Self; 3] = [Self::Pixel, Self::Shockwave, Self::Firework];

    pub fn name(self) -> &'static str {
        match self {
            Self::Pixel => "PIXEL",
            Self::Shockwave => "SHOCKWAVE",
            Self::Firework => "FIREWORK",
        }
    }

    pub fn unlock(self) -> Unlock {
        match self {
            Self::Pixel => Unlock::Free,
            Self::Shockwave => Unlock::Games(25),
            Self::Firework => Unlock::Games(250),
        }
    }
}

/// What opens a skin: nothing, a best level (D24) or games played ([114], D39).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlock {
    Free,
    Level(usize),
    Games(u64),
}

impl Unlock {
    pub fn open(self, save: &Save) -> bool {
        match self {
            Self::Free => true,
            Self::Level(level) => level <= save.best_level,
            Self::Games(games) => games <= save.stats.games(),
        }
    }

    /// What a locked skin still needs, for example "18 / 25 GAMES".
    pub fn need(self, save: &Save) -> String {
        match self {
            Self::Free => String::new(),
            Self::Level(level) => format!("LEVEL {level}"),
            Self::Games(games) => format!("{} / {games} GAMES", save.stats.games()),
        }
    }
}

/// Every skin, part by part and in unlock order: its part, its name and what opens it.
pub fn all_unlocks() -> Vec<(&'static str, &'static str, Unlock)> {
    let line = LineSkin::ALL.map(|s| ("LINE", s.name(), s.unlock()));
    let wall = WallSkin::ALL.map(|s| ("WALLS", s.name(), s.unlock()));
    let fill = FillSkin::ALL.map(|s| ("FILL", s.name(), s.unlock()));
    let kill = KillSkin::ALL.map(|s| ("KILL", s.name(), s.unlock()));
    line.into_iter()
        .chain(wall)
        .chain(fill)
        .chain(kill)
        .collect()
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
    all_unlocks()
        .into_iter()
        .filter(|(_, _, unlock)| matches!(*unlock, Unlock::Level(l) if old < l && l <= new))
        .map(|(part, name, _)| format!("{name} {part}"))
        .collect()
}

/// The skins that playing game `new` unlocks, after `old` games ([114]).
pub fn unlocked_by_games(old: u64, new: u64) -> Vec<String> {
    all_unlocks()
        .into_iter()
        .filter(|(_, _, unlock)| matches!(*unlock, Unlock::Games(g) if old < g && g <= new))
        .map(|(part, name, _)| format!("{name} {part}"))
        .collect()
}

/// The notice for new skins.
pub fn skin_notice(skins: &[String]) -> Option<String> {
    (!skins.is_empty()).then(|| format!("NEW SKIN: {}\nPICK IT IN SKINS", skins.join(", ")))
}

/// The notice for what a new best level unlocks: the soundtrack ([089]) and the skins.
fn unlock_notice(old: usize, new: usize) -> Option<String> {
    let mut lines = vec![];
    if old < SOUNDTRACK_LEVEL && SOUNDTRACK_LEVEL <= new {
        lines.push("SOUNDTRACK UNLOCKED\nTURN IT ON IN SETTINGS".to_string());
    }
    lines.extend(skin_notice(&unlocked_between(old, new)));
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
/// The GOLD line ([117]): a soft gold glow that pulses, a pale core, and glints that
/// twinkle along it. A thin gold line alone splits into red and green under the CRT pass.
const GOLD: Color = Color::srgb(1.6, 1.05, 0.15);
const GOLD_PULSE: Color = Color::srgb(2.2, 1.5, 0.3);
const GOLD_GLOW_WIDTH: f32 = 16.;
const GOLD_CORE: Color = Color::srgb(2.6, 2.2, 1.2);
const GOLD_CORE_WIDTH: f32 = 1.5;
const GLINT: Color = Color::srgb(3., 2.6, 1.6);
const GLINT_SPACING: f32 = 38.;
const GLINT_ARM: Vec2 = Vec2::new(11., 1.6);
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
        LineSkin::Gold => {
            let offsets = mark_offsets(length, GLINT_SPACING, cursor);
            segment.insert(Sprite {
                image: assets.glow.clone(),
                color: GOLD,
                custom_size: Some(Vec2::new(length, GOLD_GLOW_WIDTH)),
                ..default()
            });
            segment.with_children(|parent| {
                parent.spawn((
                    LineMark,
                    Sprite::from_color(GOLD_CORE, Vec2::new(length, GOLD_CORE_WIDTH)),
                    Transform::from_xyz(0., 0., 0.05),
                ));
                for d in offsets {
                    // Each glint twinkles in its own time.
                    let phase = cursor.count as f32 * 2.3;
                    cursor.count += 1;
                    parent.spawn((
                        LineMark,
                        Glint(phase),
                        Sprite::from_color(GLINT, GLINT_ARM),
                        Transform::from_xyz(length / 2. - d, 0., 0.1),
                        children![(
                            Sprite::from_color(GLINT, GLINT_ARM.yx()),
                            Transform::default()
                        )],
                    ));
                }
            });
        }
    }
    cursor.along += length;
}

/// A glint of the GOLD line, with its twinkle's phase.
#[derive(Component)]
struct Glint(f32);

/// A glint's size at `secs`: mostly small, a short flash now and then.
fn twinkle(secs: f32, phase: f32) -> f32 {
    let wave = 0.5 + 0.5 * ops::sin(secs * 5. + phase);
    0.35 + 0.85 * ops::powf(wave, 4.)
}

/// Draws only.
fn twinkle_glints(time: Res<Time>, mut glints: Query<(&Glint, &mut Transform)>) {
    let secs = time.elapsed_secs();
    for (glint, mut transform) in &mut glints {
        transform.scale = Vec3::splat(twinkle(secs, glint.0));
    }
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

/// A sprite of a kill effect ([116]). From `delay` seconds after the kill, over `secs`, its
/// size goes from `from` to `to` and it fades; it flies at `velocity`, which slows and falls.
#[derive(Component)]
struct KillPart {
    age: f32,
    delay: f32,
    secs: f32,
    from: Vec2,
    to: Vec2,
    color: Color,
    velocity: Vec2,
    gravity: f32,
}

impl KillPart {
    fn new(secs: f32, from: Vec2, to: Vec2, color: Color) -> Self {
        Self {
            age: 0.,
            delay: 0.,
            secs,
            from,
            to,
            color,
            velocity: Vec2::ZERO,
            gravity: 0.,
        }
    }
}

const KILL_Z: f32 = 2.;
/// The two rings: (delay, seconds, end diameter), from `WAVE_START` across.
const WAVES: [(f32, f32, f32); 2] = [(0., 0.45, 150.), (0.1, 0.4, 95.)];
const WAVE_START: f32 = 12.;
/// Two shells of streaks: (count, speed, start size); each streak shrinks to its end size.
const SHELLS: [(usize, f32, Vec2); 2] = [
    (16, 330., Vec2::new(14., 3.)),
    (8, 170., Vec2::new(10., 3.)),
];
const STREAK_END: Vec2 = Vec2::new(3., 2.);
const STREAK_SECS: f32 = 0.8;
const STREAK_DRAG: f32 = 2.5;
const STREAK_GRAVITY: f32 = 320.;
const FIREWORK_COLORS: [Color; 4] = [SUPER_NEON_CYAN, NEON_YELLOW, NEON_ORANGE, NEON_GREEN];

/// The look of a kill at `at` in `color` for a skin other than the pixel burst, which the
/// enemies module spawns. Look only: no physics and no `GameRng`.
pub fn spawn_kill(
    commands: &mut Commands,
    skin: KillSkin,
    assets: &SkinAssets,
    at: Vec2,
    color: Color,
) {
    let mut parts = vec![];
    match skin {
        KillSkin::Pixel => {}
        KillSkin::Shockwave => {
            for (delay, secs, size) in WAVES {
                let part = KillPart::new(secs, Vec2::splat(WAVE_START), Vec2::splat(size), color);
                parts.push((KillPart { delay, ..part }, Some(assets.wave.clone())));
            }
        }
        KillSkin::Firework => {
            for (shell, (count, speed, size)) in SHELLS.into_iter().enumerate() {
                for k in 0..count {
                    // The inner shell sits between the outer shell's streaks.
                    let angle =
                        std::f32::consts::TAU * (k as f32 + shell as f32 * 0.5) / count as f32;
                    let color = FIREWORK_COLORS[(k + shell) % FIREWORK_COLORS.len()];
                    let part = KillPart::new(STREAK_SECS, size, STREAK_END, color);
                    parts.push((
                        KillPart {
                            velocity: Vec2::from_angle(angle) * speed,
                            gravity: STREAK_GRAVITY,
                            ..part
                        },
                        None,
                    ));
                }
            }
        }
    }
    for (part, image) in parts {
        let sprite = Sprite {
            image: image.unwrap_or_default(),
            color: part.color,
            custom_size: Some(part.from),
            ..default()
        };
        let angle = part.velocity.to_angle();
        commands.spawn((
            DespawnOnExit(Screen::Gameplay),
            sprite,
            Transform::from_translation(at.extend(KILL_Z))
                .with_rotation(Quat::from_rotation_z(angle)),
            if part.delay > 0. {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
            part,
        ));
    }
}

/// Draws only.
fn animate_kill_parts(
    mut commands: Commands,
    time: Res<Time>,
    mut parts: Query<(
        Entity,
        &mut KillPart,
        &mut Sprite,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let dt = time.delta_secs();
    for (entity, mut part, mut sprite, mut transform, mut visibility) in &mut parts {
        part.age += dt;
        let t = (part.age - part.delay) / part.secs;
        if t < 0. {
            continue;
        }
        if t >= 1. {
            commands.entity(entity).despawn();
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        let eased = EaseFunction::QuadraticOut.sample_clamped(t);
        sprite.custom_size = Some(part.from.lerp(part.to, eased));
        sprite.color = part.color.with_alpha(1. - t * t);
        if part.velocity != Vec2::ZERO {
            let gravity = part.gravity;
            part.velocity *= ops::exp(-STREAK_DRAG * dt);
            part.velocity.y -= gravity * dt;
            transform.translation += (part.velocity * dt).extend(0.);
            transform.rotation = Quat::from_rotation_z(part.velocity.to_angle());
        }
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
        WallSkin::Hazard => assets.hazard.clone(),
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
    hazard: Handle<Image>,
    ring: Handle<Mesh>,
    steel: Handle<ColorMaterial>,
    dot: Handle<Mesh>,
    hot: Handle<ColorMaterial>,
    glow: Handle<Image>,
    /// SCANLINES, GRID and STARS.
    fills: [Handle<ColorMaterial>; 3],
    /// A soft white ring, the SHOCKWAVE kill's.
    wave: Handle<Image>,
}

impl SkinAssets {
    /// The material of a claimed shape in `skin`; the solid fill keeps the shapes' own.
    pub fn fill(&self, skin: FillSkin) -> Option<Handle<ColorMaterial>> {
        match skin {
            FillSkin::Solid => None,
            FillSkin::Scanlines => Some(self.fills[0].clone()),
            FillSkin::Grid => Some(self.fills[1].clone()),
            FillSkin::Stars => Some(self.fills[2].clone()),
        }
    }
}

impl FromWorld for SkinAssets {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        let brick = images.add(tile(&BRICK));
        let circuit = images.add(tile(&CIRCUIT));
        let hazard = images.add(tile(&HAZARD));
        let glow = images.add(glow_strip());
        let wave = images.add(wave_ring());
        let fills = [scanlines(), grid(), stars()].map(|image| {
            let size = image.size().as_vec2();
            (images.add(image), size)
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let ring = meshes.add(Annulus::new(2.5, 4.5));
        let dot = meshes.add(Circle::new(TIP_RADIUS));
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let steel = materials.add(STEEL);
        let hot = materials.add(LASER_CORE);
        let fills = fills.map(|(image, size)| {
            materials.add(ColorMaterial {
                texture: Some(image),
                uv_transform: Affine2::from_scale(1. / size),
                ..default()
            })
        });
        Self {
            brick,
            circuit,
            hazard,
            ring,
            steel,
            dot,
            hot,
            glow,
            fills,
            wave,
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

/// Yellow and black stripes ([117]).
const HAZARD: [&str; 16] = [
    "YYYYYYYYkkkkkkkk",
    "YYYYYYYkkkkkkkkY",
    "YYYYYYkkkkkkkkYY",
    "YYYYYkkkkkkkkYYY",
    "YYYYkkkkkkkkYYYY",
    "YYYkkkkkkkkYYYYY",
    "YYkkkkkkkkYYYYYY",
    "YkkkkkkkkYYYYYYY",
    "kkkkkkkkYYYYYYYY",
    "kkkkkkkYYYYYYYYk",
    "kkkkkkYYYYYYYYkk",
    "kkkkkYYYYYYYYkkk",
    "kkkkYYYYYYYYkkkk",
    "kkkYYYYYYYYkkkkk",
    "kkYYYYYYYYkkkkkk",
    "kYYYYYYYYkkkkkkk",
];

fn texel(c: char) -> [u8; 4] {
    match c {
        '=' => [255, 90, 190, 255],
        'Y' => [255, 196, 40, 255],
        'k' => [34, 28, 18, 255],
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

/// A fill's pattern: full where 255, dimmer below.
const FILL_DIM: u8 = 140;
const STARS_DIM: u8 = 150;

/// A fill tile of `size` pixels, one per world unit, repeated over the claimed shapes; its
/// gray scales the claim's color. Linear: a phone draws a world unit over 2 to 3 pixels, and
/// nearest gives lines of uneven width there.
fn fill_tile(size: UVec2, gray: impl Fn(u32, u32) -> u8) -> Image {
    let data = (0..size.y)
        .flat_map(|y| (0..size.x).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            let v = gray(x, y);
            [v, v, v, 255]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

/// Bands 5 units high, every 10.
fn scanlines() -> Image {
    fill_tile(UVec2::new(1, 10), |_, y| if y < 5 { 255 } else { FILL_DIM })
}

/// Lines 2 units wide, every 16.
fn grid() -> Image {
    fill_tile(UVec2::splat(16), |x, y| {
        if x < 2 || y < 2 { 255 } else { FILL_DIM }
    })
}

/// Bright stars on a darker fill: (x, y, big) in a 64 unit tile; a small one is a cross of
/// 5 pixels, a big one a diamond of 13.
const STARS: [(u32, u32, bool); 12] = [
    (5, 9, true),
    (22, 3, false),
    (40, 14, false),
    (55, 6, true),
    (13, 30, false),
    (31, 25, true),
    (50, 33, false),
    (3, 50, false),
    (24, 46, false),
    (44, 52, true),
    (60, 44, false),
    (35, 60, false),
];

fn stars() -> Image {
    fill_tile(UVec2::splat(64), |x, y| {
        let lit = STARS.iter().any(|&(sx, sy, big)| {
            let (dx, dy) = (x.abs_diff(sx), y.abs_diff(sy));
            dx + dy <= if big { 2 } else { 1 }
        });
        if lit { 255 } else { STARS_DIM }
    })
}

/// A white ring that fills a square image, soft at its edges.
fn wave_ring() -> Image {
    const SIDE: u32 = 128;
    let data = (0..SIDE * SIDE)
        .flat_map(|i| {
            let at = Vec2::new((i % SIDE) as f32, (i / SIDE) as f32) + 0.5;
            let r = at.distance(Vec2::splat(SIDE as f32 / 2.)) / (SIDE as f32 / 2.);
            let d = (r - 0.9) / 0.06;
            let alpha = ops::exp(-d * d);
            [255, 255, 255, (alpha * 255.) as u8]
        })
        .collect();
    let mut image = Image::new(
        Extent3d {
            width: SIDE,
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
        let open: Vec<_> = all_unlocks()
            .into_iter()
            .filter(|(_, _, unlock)| unlock.open(&save))
            .map(|(_, name, _)| name)
            .collect();
        assert_eq!(open, ["NEON", "PLAIN", "SOLID", "PIXEL"]);
    }

    #[test]
    fn looks_unlock_at_5_25_100_250_and_500_games() {
        assert_eq!(unlocked_by_games(3, 4), Vec::<String>::new());
        assert_eq!(unlocked_by_games(4, 5), ["SCANLINES FILL"]);
        assert_eq!(unlocked_by_games(5, 5), Vec::<String>::new(), "once");
        assert_eq!(unlocked_by_games(24, 25), ["SHOCKWAVE KILL"]);
        assert_eq!(unlocked_by_games(99, 100), ["GRID FILL"]);
        assert_eq!(unlocked_by_games(249, 250), ["FIREWORK KILL"]);
        assert_eq!(unlocked_by_games(499, 500), ["STARS FILL"]);
        assert_eq!(unlocked_by_games(49, 50), ["HAZARD WALLS"]);
        assert_eq!(unlocked_by_games(999, 1000), ["GOLD LINE"]);
        assert_eq!(unlocked_by_games(0, 1000).len(), 7);
        assert_eq!(
            skin_notice(&unlocked_by_games(4, 5)).as_deref(),
            Some("NEW SKIN: SCANLINES FILL\nPICK IT IN SKINS")
        );
    }

    #[test]
    fn a_locked_look_tells_what_it_needs() {
        let mut save = Save::default();
        save.stats.games = Some(18);
        assert_eq!(KillSkin::Shockwave.unlock().need(&save), "18 / 25 GAMES");
        assert!(!KillSkin::Shockwave.unlock().open(&save));
        assert!(FillSkin::Scanlines.unlock().open(&save));
        assert_eq!(LineSkin::Chain.unlock().need(&save), "LEVEL 15");
    }

    #[test]
    fn next_steps_through_the_open_skins_only() {
        let open = |best: usize| {
            move |s: LineSkin| {
                s.unlock().open(&Save {
                    best_level: best,
                    ..default()
                })
            }
        };
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
    fn a_glint_is_small_with_a_flash_now_and_then() {
        let sizes: Vec<f32> = (0..200).map(|i| twinkle(i as f32 * 0.01, 0.)).collect();
        let (min, max) = sizes
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &s| (a.min(s), b.max(s)));
        assert!(min >= 0.35 - 1e-4 && max > 1.1 && max <= 1.2 + 1e-4);
        assert!(sizes.iter().filter(|&&s| s < 0.6).count() > sizes.len() / 2);
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
    fn a_kill_effect_plays_then_goes() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<SkinAssets>();
        world.init_resource::<Time>();
        let parts = |world: &mut World| world.query::<&KillPart>().iter(world).count();
        for (skin, count) in [
            (KillSkin::Pixel, 0),
            (KillSkin::Shockwave, 2),
            (KillSkin::Firework, 24),
        ] {
            world
                .run_system_once(move |mut commands: Commands, assets: Res<SkinAssets>| {
                    spawn_kill(&mut commands, skin, &assets, Vec2::ZERO, Color::WHITE);
                })
                .unwrap();
            assert_eq!(parts(&mut world), count, "{skin:?}");
            for _ in 0..12 {
                world
                    .resource_mut::<Time>()
                    .advance_by(std::time::Duration::from_millis(100));
                world.run_system_once(animate_kill_parts).unwrap();
            }
            assert_eq!(parts(&mut world), 0, "{skin:?} goes in 1.2 s");
        }
    }

    #[test]
    fn a_save_without_a_fill_keeps_the_solid_fill() {
        let skins: Skins = serde_json::from_str(r#"{"line":"Laser","wall":"Brick"}"#).unwrap();
        assert_eq!(skins.fill, FillSkin::Solid);
        assert_eq!(skins.kill, KillSkin::Pixel);
        assert_eq!(skins.line, LineSkin::Laser);
    }

    #[test]
    fn fill_tiles_repeat_their_pattern() {
        let gray = |image: &Image, x: u32, y: u32| {
            let width = image.size().x;
            image.data.as_ref().unwrap()[((y * width + x) * 4) as usize]
        };
        for image in [scanlines(), grid(), stars()] {
            let ImageSampler::Descriptor(sampler) = &image.sampler else {
                panic!("no sampler");
            };
            assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
            assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
        }
        let lines = scanlines();
        assert_eq!((gray(&lines, 0, 4), gray(&lines, 0, 5)), (255, FILL_DIM));
        let grid = grid();
        assert_eq!((gray(&grid, 1, 9), gray(&grid, 9, 9)), (255, FILL_DIM));
        let stars = stars();
        let lit = (0..64)
            .flat_map(|y| (0..64).map(move |x| (x, y)))
            .filter(|&(x, y)| gray(&stars, x, y) == 255)
            .count();
        assert_eq!(lit, 4 * 13 + 8 * 5, "four diamonds and eight crosses");
    }

    #[test]
    fn a_tile_is_32_pixels_from_its_pattern() {
        assert!(
            [BRICK, CIRCUIT, HAZARD]
                .iter()
                .flatten()
                .all(|row| row.len() == 16)
        );
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
