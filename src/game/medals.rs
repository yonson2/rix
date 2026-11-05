//! Medals ([111], D39): feats only Rix has. A feat triggers `Earn`; the first time, and not
//! in the first run (D27), the medal goes into the save and a notice shows. The RECORDS
//! screen lists them. Medals only watch the game: no draws from `GameRng`.

use bevy::prelude::*;

use super::{
    events::{DespawnEnemy, DrawPolygon},
    items::star::Star,
    level::DespawnAfter,
    setup::{LevelProgress, Run},
    state::Phase,
    tutorial::{KINDS, Kind},
    wall::Field,
};
use crate::{
    save::Save,
    screens::Screen,
    theme::{FontAssets, palette::NEON_YELLOW},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LevelTally>()
        .add_observer(on_earn)
        .add_observer(count_star_kill)
        .add_systems(
            OnEnter(Phase::Playing),
            (start_level, reach_level, meet_every_enemy),
        )
        .add_systems(OnEnter(Phase::LevelTransition), clear_level)
        .add_systems(
            Update,
            (
                count_claims,
                meet_every_enemy.run_if(resource_changed::<Save>),
            )
                .run_if(in_state(Phase::Playing)),
        );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Medal {
    OneStroke,
    Feast,
    Pacifist,
    StarStrike,
    BombSquad,
    LandGrab,
    KnowYourEnemies,
    DeepDive,
}

impl Medal {
    pub const ALL: [Self; 8] = [
        Self::OneStroke,
        Self::Feast,
        Self::Pacifist,
        Self::StarStrike,
        Self::BombSquad,
        Self::LandGrab,
        Self::KnowYourEnemies,
        Self::DeepDive,
    ];

    /// The medal's id in the save.
    pub fn id(self) -> String {
        format!("{self:?}")
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::OneStroke => "ONE STROKE",
            Self::Feast => "FEAST",
            Self::Pacifist => "PACIFIST",
            Self::StarStrike => "STAR STRIKE",
            Self::BombSquad => "BOMB SQUAD",
            Self::LandGrab => "LAND GRAB",
            Self::KnowYourEnemies => "KNOW YOUR ENEMIES",
            Self::DeepDive => "DEEP DIVE",
        }
    }

    pub fn rule(self) -> &'static str {
        match self {
            Self::OneStroke => "CLEAR A LEVEL WITH ONE CLAIM",
            Self::Feast => "EAT 3 ENEMIES WITH ONE CLAIM",
            Self::Pacifist => "CLEAR A LEVEL WITH NO KILLS",
            Self::StarStrike => "KILL 5 ENEMIES IN ONE STAR",
            Self::BombSquad => "BOMB 3 ENEMIES IN ONE LEVEL",
            Self::LandGrab => "CLEAR A LEVEL AT 90 %",
            Self::KnowYourEnemies => "MEET EVERY ENEMY",
            Self::DeepDive => "REACH LEVEL 25",
        }
    }
}

/// One claim eats this many enemies for FEAST.
pub const FEAST_EATEN: usize = 3;
const STAR_STRIKE_KILLS: usize = 5;
const BOMB_SQUAD_BOMBS: usize = 3;
/// The claimed fraction for LAND GRAB; the margin takes 0.89999 as 90 %.
const LAND_GRAB_CLAIMED: f32 = 0.9 - 1e-4;
const DEEP_DIVE_LEVEL: usize = 25;
const NOTICE_SECS: f32 = 3.5;
/// Above the skin unlock notice; a second medal at once shows above the first.
const NOTICE_Y: f32 = 190.;
const NOTICE_GAP: f32 = 30.;

/// A feat happened; `on_earn` keeps the first of each medal.
#[derive(Event, Clone, Copy, Debug)]
pub struct Earn(pub Medal);

/// What the level in progress has done, for the medals of a level clear.
#[derive(Resource, Default, Debug)]
pub struct LevelTally {
    claims: usize,
    kills_at_start: usize,
    bombs: usize,
    star_kills: usize,
}

impl LevelTally {
    /// A bomb took an enemy; true when it makes BOMB SQUAD.
    pub fn bombed(&mut self) -> bool {
        self.bombs += 1;
        self.bombs == BOMB_SQUAD_BOMBS
    }
}

#[derive(Component)]
struct MedalNotice;

fn on_earn(
    earn: On<Earn>,
    mut commands: Commands,
    mut save: ResMut<Save>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
    notices: Query<(), With<MedalNotice>>,
) {
    let id = earn.0.id();
    // The first run, the one with the tutorial, earns nothing (D27, [092]).
    if save.stats.runs == 0 || save.medals.contains(&id) {
        return;
    }
    save.medals.push(id);
    let y =
        field.inner_corners().bottom_left.y + NOTICE_Y + NOTICE_GAP * notices.iter().count() as f32;
    commands.spawn((
        MedalNotice,
        DespawnOnExit(Screen::Gameplay),
        DespawnAfter::from_seconds(NOTICE_SECS),
        Text2d::new(format!("MEDAL: {}", earn.0.name())),
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

/// The medals of a level clear: its claims, its kills and the claimed fraction.
fn clear_medals(claims: usize, kills: usize, claimed: f32) -> Vec<Medal> {
    let mut medals = vec![];
    if claims == 1 {
        medals.push(Medal::OneStroke);
    }
    if kills == 0 {
        medals.push(Medal::Pacifist);
    }
    if claimed >= LAND_GRAB_CLAIMED {
        medals.push(Medal::LandGrab);
    }
    medals
}

fn start_level(mut tally: ResMut<LevelTally>, run: Res<Run>) {
    *tally = LevelTally {
        kills_at_start: run.kill_count,
        ..default()
    };
}

fn count_claims(mut claims: MessageReader<DrawPolygon>, mut tally: ResMut<LevelTally>) {
    tally.claims += claims.read().count();
}

fn clear_level(
    mut commands: Commands,
    tally: Res<LevelTally>,
    run: Res<Run>,
    progress: Res<LevelProgress>,
) {
    let kills = run.kill_count - tally.kills_at_start;
    for medal in clear_medals(tally.claims, kills, progress.current) {
        commands.trigger(Earn(medal));
    }
}

fn reach_level(mut commands: Commands, run: Res<Run>) {
    if run.level >= DEEP_DIVE_LEVEL {
        commands.trigger(Earn(Medal::DeepDive));
    }
}

/// True when the tips met every enemy kind ([070]).
fn every_enemy_met(met: &[String]) -> bool {
    KINDS
        .iter()
        .filter(|kind| matches!(kind, Kind::Enemy(_)))
        .all(|kind| met.contains(&kind.id()))
}

fn meet_every_enemy(mut commands: Commands, save: Res<Save>) {
    if every_enemy_met(&save.met) && !save.medals.contains(&Medal::KnowYourEnemies.id()) {
        commands.trigger(Earn(Medal::KnowYourEnemies));
    }
}

/// A kill while the Star lasts counts toward STAR STRIKE; a new Star counts from 0.
fn count_star_kill(
    _: On<DespawnEnemy>,
    mut commands: Commands,
    star: Option<Res<Star>>,
    mut tally: ResMut<LevelTally>,
) {
    let Some(star) = star else {
        return;
    };
    if star.is_added() {
        tally.star_kills = 0;
    }
    tally.star_kills += 1;
    if tally.star_kills == STAR_STRIKE_KILLS {
        commands.trigger(Earn(Medal::StarStrike));
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn a_level_clear_earns_by_its_claims_kills_and_claimed_area() {
        use Medal::*;
        assert_eq!(clear_medals(1, 0, 0.8), [OneStroke, Pacifist]);
        assert_eq!(clear_medals(2, 3, 0.8), []);
        assert_eq!(clear_medals(3, 1, 0.89999), [LandGrab]);
        assert_eq!(clear_medals(1, 0, 0.95), [OneStroke, Pacifist, LandGrab]);
    }

    #[test]
    fn the_third_bomb_of_a_level_makes_bomb_squad_once() {
        let mut tally = LevelTally::default();
        assert_eq!(
            [
                tally.bombed(),
                tally.bombed(),
                tally.bombed(),
                tally.bombed()
            ],
            [false, false, true, false]
        );
    }

    #[test]
    fn every_enemy_kind_must_be_met() {
        let enemies: Vec<String> = KINDS
            .iter()
            .filter(|kind| matches!(kind, Kind::Enemy(_)))
            .map(|kind| kind.id())
            .collect();
        assert_eq!(enemies.len(), 5);
        assert!(every_enemy_met(&enemies));
        assert!(!every_enemy_met(&enemies[1..]));
        assert!(every_enemy_met(&crate::game::tutorial::all_met()));
    }

    #[test]
    fn medal_ids_are_unique() {
        let mut ids: Vec<String> = Medal::ALL.map(Medal::id).to_vec();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), Medal::ALL.len());
    }

    fn app(runs: u64) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Font>()
            .insert_resource(FontAssets {
                main: Handle::default(),
            })
            .init_resource::<Field>()
            .init_resource::<LevelTally>()
            .add_observer(on_earn)
            .add_observer(count_star_kill);
        let mut save = Save::default();
        save.stats.runs = runs;
        app.insert_resource(save);
        app
    }

    fn earned(app: &App) -> Vec<String> {
        app.world().resource::<Save>().medals.clone()
    }

    #[test]
    fn a_medal_is_earned_once_with_a_notice_and_never_in_the_first_run() {
        let mut first = app(0);
        first.world_mut().trigger(Earn(Medal::Feast));
        assert_eq!(earned(&first), Vec::<String>::new(), "the first run");

        // As a level clear does: several medals from one system.
        let mut app = app(1);
        app.world_mut()
            .run_system_once(|mut commands: Commands| {
                commands.trigger(Earn(Medal::Feast));
                commands.trigger(Earn(Medal::Feast));
                commands.trigger(Earn(Medal::DeepDive));
            })
            .unwrap();
        app.world_mut().flush();
        assert_eq!(earned(&app), ["Feast", "DeepDive"]);
        let notices = app
            .world_mut()
            .query_filtered::<&Transform, With<MedalNotice>>()
            .iter(app.world())
            .map(|t| t.translation.y)
            .collect::<Vec<_>>();
        assert_eq!(notices.len(), 2, "one notice per new medal");
        assert_ne!(notices[0], notices[1], "two notices do not overlap");
    }

    #[test]
    fn five_kills_in_one_star_make_star_strike() {
        let mut app = app(1);
        let kill = |app: &mut App| {
            app.world_mut().trigger(DespawnEnemy(Entity::PLACEHOLDER));
            app.update();
        };
        for _ in 0..4 {
            kill(&mut app);
        }
        assert_eq!(earned(&app), Vec::<String>::new(), "no Star, no count");

        app.world_mut().insert_resource(Star::default());
        for _ in 0..4 {
            kill(&mut app);
        }
        assert_eq!(earned(&app), Vec::<String>::new());
        // A new Star counts from 0.
        app.world_mut().remove_resource::<Star>();
        app.world_mut().insert_resource(Star::default());
        for _ in 0..4 {
            kill(&mut app);
        }
        assert_eq!(earned(&app), Vec::<String>::new());
        kill(&mut app);
        assert_eq!(earned(&app), ["StarStrike"]);
    }
}
