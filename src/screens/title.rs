//! The title screen. Its actions come in tiers by purpose ([127]): PLAY leads (the primary
//! button), CHALLENGES follows as a card with the day's challenge, the collections (SKINS,
//! RECORDS, WIKI) are three tiles, and the utilities sit at the edges: a gear for SETTINGS,
//! CREDITS (and QUIT on desktop) as plain text at the bottom.

use bevy::prelude::*;

use crate::{
    common::Transition,
    game::{challenges, setup::Points},
    save::Save,
    screens::Screen,
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{
            CYAN, GREEN, LIGHT_PURPLE, NEON_CYAN, NEON_GRAY, NEON_GREEN, NEON_PURPLE, NEON_WHITE,
            NEON_YELLOW, PURPLE, YELLOW,
        },
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), setup);
    app.add_systems(
        Update,
        (flicker_logo, breathe_logo).run_if(in_state(Screen::Title)),
    );
}

/// Three groups, a wider space between them: the logo and the best score, the two ways to
/// play, the collections; then the footer.
const LOGO_Y: f32 = 230.;
const LOGO_FONT: f32 = 104.;
const BEST_Y: f32 = 120.;
const PLAY_Y: f32 = 36.;
const PLAY_SIZE: Vec2 = Vec2::new(320., 104.);
const CHALLENGES_Y: f32 = -74.;
const CHALLENGES_SIZE: Vec2 = Vec2::new(320., 84.);
const TILES_Y: f32 = -210.;
const TILE_SIZE: Vec2 = Vec2::new(100., 100.);
/// From one tile's centre to the next: the row is as wide as PLAY.
const TILE_STEP: f32 = 110.;
const FOOTER_Y: f32 = -310.;
const FOOTER_SIZE: Vec2 = Vec2::new(130., 48.);
/// The gear sits in the top corner, a thumb's reach from the edge.
const GEAR_AT: Vec2 = Vec2::new(176., 352.);
const GEAR_SIZE: Vec2 = Vec2::new(60., 60.);

/// The "RIX" box; its text is a child.
#[derive(Component)]
struct Logo;

/// A collection tile: its screen, its label and its icon.
const TILES: [(Screen, &str, Icon); 3] = [
    (Screen::Skins, "SKINS", Icon::Palette),
    (Screen::Records, "RECORDS", Icon::Trophy),
    (Screen::Wiki, "WIKI", Icon::Book),
];

#[derive(Clone, Copy)]
enum Icon {
    Palette,
    Trophy,
    Book,
}

fn text(font: &Handle<Font>, text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text2d::new(text),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn go(screen: Screen) -> impl Fn(On<Activate>, ResMut<NextState<Screen>>) + Send + Sync + 'static {
    move |_, mut next| next.set(screen)
}

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font = &fonts.main;
    let shape = Shape::new(shape::rounded_rectangle(
        Vec2::new(LOGO_FONT * 2., LOGO_FONT * 1.5),
        10.,
    ))
    .fill(PURPLE)
    .stroke(NEON_PURPLE, 3.0);
    commands.spawn((
        Logo,
        DespawnOnExit(Screen::Title),
        shape,
        Transform::from_xyz(0., LOGO_Y, 0.),
        children![(
            text(font, "RIX", LOGO_FONT, Color::NONE),
            Transform::from_xyz(0., 0., 1.),
            Transition(Timer::from_seconds(2.6, TimerMode::Once)),
        )],
    ));
    if save.best > 0 {
        commands.spawn((
            DespawnOnExit(Screen::Title),
            text(font, format!("BEST {}", Points(save.best)), 22., NEON_WHITE),
            Transform::from_xyz(0., BEST_Y, 1.),
        ));
    }

    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::primary_button("PLAY", font.clone(), Vec2::new(0., PLAY_Y), PLAY_SIZE),
        ))
        .observe(go(Screen::Gameplay));

    let today = challenges::today();
    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::box_button(Vec2::new(0., CHALLENGES_Y), CHALLENGES_SIZE),
        ))
        .observe(go(Screen::Challenges))
        .with_children(|card| {
            card.spawn((
                text(font, "CHALLENGES", 28., NEON_WHITE),
                Transform::from_xyz(0., 12., 0.1),
            ));
            card.spawn((
                text(font, daily_line(&save, today), 15., NEON_YELLOW),
                Transform::from_xyz(0., -20., 0.1),
            ));
        });

    for (i, (screen, label, icon)) in TILES.into_iter().enumerate() {
        let x = (i as f32 - 1.) * TILE_STEP;
        commands
            .spawn((
                DespawnOnExit(Screen::Title),
                widget::box_button(Vec2::new(x, TILES_Y), TILE_SIZE),
            ))
            .observe(go(screen))
            .with_children(|tile| {
                for (shape, at) in icon_parts(icon) {
                    tile.spawn((
                        shape,
                        Transform::from_translation((at + ICON_AT).extend(0.1)),
                    ));
                }
                tile.spawn((
                    text(font, label, 18., NEON_WHITE),
                    Transform::from_xyz(0., -30., 0.1),
                ));
            });
    }

    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::text_button("", font.clone(), GEAR_AT, GEAR_SIZE),
        ))
        .observe(go(Screen::Settings))
        .with_children(|gear| {
            for (shape, at) in gear_parts() {
                gear.spawn((shape, Transform::from_translation(at.extend(0.1))));
            }
        });

    // Phones and the web close apps their own way: there CREDITS stands alone.
    #[cfg(any(target_os = "ios", target_os = "android", target_family = "wasm"))]
    let credits_x = 0.;
    #[cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))]
    let credits_x = -FOOTER_SIZE.x / 2.;
    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::text_button(
                "CREDITS",
                font.clone(),
                Vec2::new(credits_x, FOOTER_Y),
                FOOTER_SIZE,
            ),
        ))
        .observe(go(Screen::Credits));
    #[cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))]
    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::text_button(
                "QUIT",
                font.clone(),
                Vec2::new(FOOTER_SIZE.x / 2., FOOTER_Y),
                FOOTER_SIZE,
            ),
        ))
        .observe(quit);
}

/// The CHALLENGES card's second line: the day's challenge, and the days in a row.
fn daily_line(save: &Save, today: u64) -> String {
    let name = challenges::daily(today).name;
    match save.daily.streak_on(today) {
        0 => format!("DAILY {name}"),
        streak => format!("DAILY {name}   STREAK {streak}"),
    }
}

/// An icon's centre in its tile, over the label.
const ICON_AT: Vec2 = Vec2::new(0., 12.);

/// Neon line icons of about 36 units, as shapes and their offsets.
fn icon_parts(icon: Icon) -> Vec<(Shape, Vec2)> {
    let dot = |color: Color, light: Color| {
        Shape::new(shape::rounded_rectangle(Vec2::splat(18.), 9.))
            .fill(color.with_alpha(0.6))
            .stroke(light, 2.5)
    };
    match icon {
        // Three paint dots.
        Icon::Palette => vec![
            (dot(CYAN, NEON_CYAN), Vec2::new(-9., 6.)),
            (dot(YELLOW, NEON_YELLOW), Vec2::new(9., 6.)),
            (dot(LIGHT_PURPLE, NEON_PURPLE), Vec2::new(0., -9.)),
        ],
        // A cup on a stem and a base.
        Icon::Trophy => {
            let part = |points: &[Vec2]| {
                Shape::new(shape::rounded_polygon(points, 2.))
                    .fill(YELLOW.with_alpha(0.3))
                    .stroke(NEON_YELLOW, 2.5)
            };
            let cup = [(-13., 16.), (13., 16.), (8., -1.), (-8., -1.)];
            let stem = [(-2.5, -1.), (2.5, -1.), (2.5, -9.), (-2.5, -9.)];
            let base = [(-9., -9.), (9., -9.), (9., -15.), (-9., -15.)];
            [cup, stem, base]
                .into_iter()
                .map(|points| (part(&points.map(|(x, y)| Vec2::new(x, y))), Vec2::ZERO))
                .collect()
        }
        // An open book: two pages from the spine.
        Icon::Book => [-1., 1.]
            .into_iter()
            .map(|side: f32| {
                let points = [(1., -12.), (1., 13.), (17., 16.), (17., -9.)]
                    .map(|(x, y)| Vec2::new(side * x, y));
                let page = Shape::new(shape::rounded_polygon(&points, 2.))
                    .fill(GREEN.with_alpha(0.3))
                    .stroke(NEON_GREEN, 2.5);
                (page, Vec2::ZERO)
            })
            .collect(),
    }
}

/// A gear of eight teeth with a hole.
fn gear_parts() -> Vec<(Shape, Vec2)> {
    const TEETH: usize = 8;
    const OUTER: f32 = 17.;
    const INNER: f32 = 12.5;
    let pitch = std::f32::consts::TAU / TEETH as f32;
    let points: Vec<Vec2> = (0..TEETH)
        .flat_map(|tooth| {
            let at = tooth as f32 * pitch;
            [(-0.36, INNER), (-0.2, OUTER), (0.2, OUTER), (0.36, INNER)].map(|(da, radius)| {
                let angle = at + da * pitch * 1.4;
                Vec2::new(ops::cos(angle), ops::sin(angle)) * radius
            })
        })
        .collect();
    let gear = Shape::new(shape::rounded_polygon(&points, 1.))
        .fill(NEON_GRAY.with_alpha(0.12))
        .stroke(NEON_GRAY, 2.5);
    let hole = Shape::new(shape::rounded_rectangle(Vec2::splat(10.), 5.)).stroke(NEON_GRAY, 2.5);
    vec![(gear, Vec2::ZERO), (hole, Vec2::ZERO)]
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))]
fn quit(_: On<Activate>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

/// The logo text and outline flicker in.
fn flicker_logo(
    mut commands: Commands,
    time: Res<Time>,
    mut text_q: Query<(Entity, &mut Transition, &mut TextColor)>,
    mut shape_q: Query<&mut Shape, With<Logo>>,
) {
    for (entity, mut transition, mut text_color) in text_q.iter_mut() {
        transition.0.tick(time.delta());
        let t = transition.0.fraction();
        let easing = EasingCurve::new(0.0, 1.0, EaseFunction::BounceInOut);
        let alpha = easing.sample(t).unwrap();
        let mut current_color = NEON_WHITE;
        current_color.set_alpha(alpha);

        text_color.0 = current_color;
        for mut shape in &mut shape_q {
            if let Some(stroke) = &mut shape.stroke {
                stroke.color = current_color;
            }
        }

        if transition.0.is_finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

/// After the flicker, the logo grows and shrinks a little, slowly.
fn breathe_logo(time: Res<Time>, mut logo: Query<&mut Transform, With<Logo>>) {
    let scale = 1. + 0.025 * ops::sin(time.elapsed_secs() * 1.6);
    for mut transform in &mut logo {
        transform.scale = Vec3::splat(scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_challenges_card_names_the_day_and_the_streak() {
        let day = 20_397;
        let mut save = Save::default();
        let name = challenges::daily(day).name;
        assert_eq!(daily_line(&save, day), format!("DAILY {name}"));
        save.daily.day = day;
        save.daily.streak = 4;
        assert_eq!(daily_line(&save, day), format!("DAILY {name}   STREAK 4"));
    }

    #[test]
    fn the_groups_do_not_overlap() {
        let bottom = |y: f32, size: Vec2| y - size.y / 2.;
        let top = |y: f32, size: Vec2| y + size.y / 2.;
        assert!(BEST_Y - 11. > top(PLAY_Y, PLAY_SIZE));
        assert!(bottom(PLAY_Y, PLAY_SIZE) > top(CHALLENGES_Y, CHALLENGES_SIZE));
        assert!(bottom(CHALLENGES_Y, CHALLENGES_SIZE) > top(TILES_Y, TILE_SIZE));
        assert!(bottom(TILES_Y, TILE_SIZE) > top(FOOTER_Y, FOOTER_SIZE));
        const {
            assert!(LOGO_Y - LOGO_FONT * 0.75 > BEST_Y + 11.);
            assert!(TILE_STEP > TILE_SIZE.x);
        }
    }
}
