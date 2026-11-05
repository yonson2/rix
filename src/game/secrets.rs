//! Secret looks ([119], D39): three feats the game never names. The first time one happens,
//! and not in the first run (D27), its look opens and a notice says so. The palindrome is
//! found at the game over. Secrets only watch the game: no draws from `GameRng`.

use bevy::prelude::*;

use super::{
    events::BuildPolygon,
    level::DespawnAfter,
    setup::LevelProgress,
    skins,
    state::Phase,
    wall::{Field, WallCorners},
};
use crate::{
    save::Save,
    theme::{FontAssets, palette::NEON_YELLOW},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::LevelTransition), find_exact)
        .add_systems(Update, find_walls.run_if(in_state(Phase::Playing)));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Secret {
    /// A level clear at 75.0 %.
    Exact,
    /// One line that touches all four walls.
    Walls,
    /// A run that ends on a palindrome score of 1,000 or more.
    Palindrome,
}

impl Secret {
    /// The secret's id in the save.
    pub fn id(self) -> String {
        format!("{self:?}")
    }
}

/// A line point this close to a wall touches it. A line ends 1 unit from the boundary, so
/// it can touch a wall on its way.
const TOUCH: f32 = 6.;
const NOTICE_SECS: f32 = 3.5;
/// Above the bottom wall, under the medals' notices ([111]).
const NOTICE_Y: f32 = 110.;

/// The claimed fraction of a clear is 75.0 % to one decimal.
pub fn exact_clear(claimed: f32) -> bool {
    (claimed * 1000.).round() == 750.
}

/// A line from wall to wall comes within `TOUCH` of each of the four walls: from a corner to
/// the far corner, or past a wall on its way.
pub fn touches_four_walls(line: &[Vec2], walls: &WallCorners) -> bool {
    let (low, high) = (walls.bottom_left, walls.top_right);
    let gaps = line.iter().fold([f32::MAX; 4], |gaps, &p| {
        [
            gaps[0].min(p.x - low.x),
            gaps[1].min(high.x - p.x),
            gaps[2].min(p.y - low.y),
            gaps[3].min(high.y - p.y),
        ]
    });
    gaps.iter().all(|&gap| gap < TOUCH)
}

/// A score of 1,000 or more that reads the same both ways, for example 12,321.
pub fn palindrome(score: u128) -> bool {
    let digits = score.to_string();
    score >= 1000 && digits.bytes().eq(digits.bytes().rev())
}

/// Keeps `secret` in the save; true the first time, never in the first run (D27, [092]).
pub fn find(save: &mut Save, secret: Secret) -> bool {
    let id = secret.id();
    if save.stats.runs == 0 || save.secrets.contains(&id) {
        return false;
    }
    save.secrets.push(id);
    true
}

/// The notice of a found secret, in three lines: "HONEYCOMB FILL" after "SECRET FOUND: "
/// is wider than the field.
pub fn notice(secret: Secret) -> String {
    format!(
        "SECRET FOUND\n{}\nPICK IT IN SKINS",
        skins::secret_look(secret)
    )
}

fn spawn_notice(commands: &mut Commands, secret: Secret, fonts: &FontAssets, field: &Field) {
    let y = field.inner_corners().bottom_left.y + NOTICE_Y;
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        DespawnAfter::from_seconds(NOTICE_SECS),
        Text2d::new(notice(secret)),
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

fn find_exact(
    mut commands: Commands,
    progress: Res<LevelProgress>,
    mut save: ResMut<Save>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
) {
    if exact_clear(progress.current) && find(&mut save, Secret::Exact) {
        spawn_notice(&mut commands, Secret::Exact, &fonts, &field);
    }
}

fn find_walls(
    mut commands: Commands,
    mut lines: MessageReader<BuildPolygon>,
    mut save: ResMut<Save>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
) {
    let walls = field.inner_corners();
    if lines.read().any(|line| touches_four_walls(line, &walls)) && find(&mut save, Secret::Walls) {
        spawn_notice(&mut commands, Secret::Walls, &fonts, &field);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clear_at_75_0_percent_is_exact() {
        assert!(exact_clear(0.75));
        assert!(exact_clear(0.7504));
        assert!(!exact_clear(0.7506), "75.1 %");
        assert!(!exact_clear(0.76));
        assert!(!exact_clear(0.9));
    }

    #[test]
    fn a_line_touches_four_walls_from_corner_to_corner_or_on_its_way() {
        let walls = Field::default().inner_corners();
        let (low, high) = (walls.bottom_left, walls.top_right);
        assert!(touches_four_walls(&[low, Vec2::ZERO, high], &walls));
        let across = [Vec2::new(low.x, 0.), Vec2::new(high.x, 0.)];
        assert!(!touches_four_walls(&across, &walls), "two walls");
        let past_both = [
            Vec2::new(low.x, 0.),
            Vec2::new(-50., high.y - 4.),
            Vec2::new(50., low.y + 4.),
            Vec2::new(high.x, 0.),
        ];
        assert!(touches_four_walls(&past_both, &walls));
        let short = [
            past_both[0],
            Vec2::new(-50., high.y - 10.),
            past_both[2],
            past_both[3],
        ];
        assert!(!touches_four_walls(&short, &walls), "10 units off the top");
    }

    #[test]
    fn a_palindrome_score_has_four_digits_or_more() {
        assert!(palindrome(12_321));
        assert!(palindrome(1001));
        assert!(palindrome(4_000_000_004));
        assert!(!palindrome(999), "below 1,000");
        assert!(!palindrome(12_345));
        assert!(!palindrome(1000));
    }

    #[test]
    fn a_secret_is_found_once_and_not_in_the_first_run() {
        let mut save = Save::default();
        assert!(!find(&mut save, Secret::Walls), "the first run");
        save.stats.runs = 1;
        assert!(find(&mut save, Secret::Walls));
        assert!(!find(&mut save, Secret::Walls), "once");
        assert_eq!(save.secrets, ["Walls"]);
    }

    #[test]
    fn a_clear_at_75_0_percent_finds_its_secret_with_a_notice() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(FontAssets {
            main: Handle::default(),
        });
        world.init_resource::<Field>();
        let mut save = Save::default();
        save.stats.runs = 1;
        world.insert_resource(save);
        for (claimed, found) in [(0.76, false), (0.7502, true)] {
            world.insert_resource(LevelProgress {
                current: claimed,
                ..default()
            });
            world.run_system_once(find_exact).unwrap();
            let notices = world.query::<&Text2d>().iter(&world).count();
            assert_eq!(notices, usize::from(found), "{claimed}");
        }
        assert_eq!(world.resource::<Save>().secrets, ["Exact"]);
        assert_eq!(
            notice(Secret::Exact),
            "SECRET FOUND\nHONEYCOMB FILL\nPICK IT IN SKINS"
        );
    }
}
