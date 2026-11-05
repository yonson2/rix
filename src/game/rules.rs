//! Challenge rules ([120], D39): a challenge is a seed whose 24 high bits hold its rules,
//! so a code gives every player the same challenge (D23). Rules of 0 are the normal game:
//! a new run's 40-bit seed ([067]) and every code so far keep their game. A challenge code
//! has up to 13 characters. The effects ([121]): the enemy kinds and count at each level
//! start, the speed on the physics clock (every enemy and bullet), the clear target and the
//! start level in `new_run`, no items, and harder rules pay more points.

use avian2d::prelude::*;
use bevy::prelude::*;

use super::enemies::rng::EnemyKind;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), set_speed);
}

/// The bits of a seed under the rules: the field.
pub const FIELD_BITS: u32 = 40;

/// The rules of the run in progress; `new_run` sets them from the seed. Each value is an
/// index, and 0 is the normal game's.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rules {
    /// The enemy kinds that spawn, one bit each in `EnemyKind` order; 0: the normal table.
    pub kinds: u8,
    /// The enemies' speed: 1, 1.25, 1.5 or 2 times.
    pub speed: u8,
    /// The enemies' count: 1, 1.5, 2 or 3 times.
    pub count: u8,
    /// `COUNTDOWNS`.
    pub countdown: u8,
    /// The clear target: 75, 60, 85 or 90 %.
    pub target: u8,
    pub no_items: bool,
    /// The start level less 1: 0 to 15.
    pub start: u8,
}

/// The countdown in seconds; 0: none.
pub const COUNTDOWNS: [u32; 6] = [0, 60, 90, 120, 180, 300];
const SPEEDS: [f32; 4] = [1., 1.25, 1.5, 2.];
const COUNTS: [f32; 4] = [1., 1.5, 2., 3.];
const TARGETS: [f32; 4] = [0.75, 0.6, 0.85, 0.9];
/// What each value pays, in percent: a harder rule pays more, an easier one less.
const SPEED_PAY: [u128; 4] = [100, 125, 150, 200];
const COUNT_PAY: [u128; 4] = [100, 125, 150, 200];
const TARGET_PAY: [u128; 4] = [100, 75, 125, 150];
const NO_ITEMS_PAY: u128 = 125;

/// The layout this build reads; a code of another version does not play.
const VERSION: u64 = 1;
/// Each value's place in the rules: (shift, width). Bits 22 and 23 are spare.
const VERSION_AT: (u32, u32) = (0, 2);
const KINDS_AT: (u32, u32) = (2, 5);
const SPEED_AT: (u32, u32) = (7, 2);
const COUNT_AT: (u32, u32) = (9, 2);
const COUNTDOWN_AT: (u32, u32) = (11, 3);
const TARGET_AT: (u32, u32) = (14, 2);
const ITEMS_AT: (u32, u32) = (16, 2);
const START_AT: (u32, u32) = (18, 4);
const USED_BITS: u32 = 22;

impl Rules {
    /// The kinds that spawn, from level 1; none: the normal table.
    pub fn kinds(self) -> Vec<EnemyKind> {
        EnemyKind::ALL
            .into_iter()
            .enumerate()
            .filter(|(bit, _)| self.kinds & 1 << bit != 0)
            .map(|(_, kind)| kind)
            .collect()
    }

    /// The enemies' speed factor.
    pub fn speed(self) -> f32 {
        SPEEDS[usize::from(self.speed)]
    }

    /// A level's enemy count under these rules, from its normal count.
    pub fn count(self, normal: usize) -> usize {
        (normal as f32 * COUNTS[usize::from(self.count)]).round() as usize
    }

    /// The claimed fraction that clears a level.
    pub fn target(self) -> f32 {
        TARGETS[usize::from(self.target)]
    }

    /// The countdown's seconds, or none ([122]).
    pub fn countdown(self) -> Option<u32> {
        Some(COUNTDOWNS[usize::from(self.countdown)]).filter(|&secs| secs > 0)
    }

    /// The level a run starts at.
    pub fn level(self) -> usize {
        usize::from(self.start) + 1
    }

    /// `points` under these rules: the pay of each value, one after the other. It saturates
    /// as the score does ([031]).
    pub fn pay(self, points: u128) -> u128 {
        let items = if self.no_items { NO_ITEMS_PAY } else { 100 };
        let percent = [
            SPEED_PAY[usize::from(self.speed)],
            COUNT_PAY[usize::from(self.count)],
            TARGET_PAY[usize::from(self.target)],
            items,
        ]
        .into_iter()
        .fold(100, |percent, pay| percent * pay / 100);
        points.checked_mul(percent).map_or(u128::MAX, |p| p / 100)
    }

    /// The rules of `seed`, or `None` for a version or a value this build does not know.
    pub fn of_seed(seed: u64) -> Option<Self> {
        let bits = seed >> FIELD_BITS;
        if bits == 0 {
            return Some(Self::default());
        }
        let value = |(shift, width): (u32, u32)| (bits >> shift) & ((1 << width) - 1);
        let known = value(VERSION_AT) == VERSION
            && (value(COUNTDOWN_AT) as usize) < COUNTDOWNS.len()
            && value(ITEMS_AT) <= 1
            && bits >> USED_BITS == 0;
        known.then(|| Self {
            kinds: value(KINDS_AT) as u8,
            speed: value(SPEED_AT) as u8,
            count: value(COUNT_AT) as u8,
            countdown: value(COUNTDOWN_AT) as u8,
            target: value(TARGET_AT) as u8,
            no_items: value(ITEMS_AT) == 1,
            start: value(START_AT) as u8,
        })
    }

    /// The seed of these rules on the field `field`; the normal rules leave it as it is.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the CHALLENGES list makes its codes with it, [123]"
        )
    )]
    pub fn seed(self, field: u64) -> u64 {
        if self == Self::default() {
            return field;
        }
        let place = |value: u8, (shift, _): (u32, u32)| u64::from(value) << shift;
        let bits = VERSION
            | place(self.kinds, KINDS_AT)
            | place(self.speed, SPEED_AT)
            | place(self.count, COUNT_AT)
            | place(self.countdown, COUNTDOWN_AT)
            | place(self.target, TARGET_AT)
            | place(u8::from(self.no_items), ITEMS_AT)
            | place(self.start, START_AT);
        bits << FIELD_BITS | field
    }
}

/// Every enemy and bullet moves at the rules' speed: the physics clock runs that much faster.
fn set_speed(rules: Res<Rules>, mut physics: ResMut<Time<Physics>>) {
    physics.set_relative_speed(rules.speed());
}

/// The items spawn unless the rules take them out.
pub fn items_on(rules: Res<Rules>) -> bool {
    !rules.no_items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::seed;

    #[test]
    fn every_40_bit_seed_plays_the_normal_rules() {
        for field in [0, 1, 12_345, (1 << FIELD_BITS) - 1] {
            assert_eq!(Rules::of_seed(field), Some(Rules::default()), "{field}");
            assert_eq!(Rules::default().seed(field), field);
        }
    }

    #[test]
    fn rules_round_trip_through_the_seed_and_its_code() {
        let field = 0xAB_CDEF_1234;
        let all = Rules {
            kinds: 0b11111,
            speed: 3,
            count: 3,
            countdown: 5,
            target: 3,
            no_items: true,
            start: 15,
        };
        for rules in [
            all,
            Rules {
                kinds: 0b00001,
                count: 2,
                ..default()
            },
            Rules {
                countdown: 1,
                ..default()
            },
            Rules {
                start: 9,
                ..default()
            },
        ] {
            let seed = rules.seed(field);
            assert_eq!(seed & ((1 << FIELD_BITS) - 1), field, "the field stays");
            assert_eq!(Rules::of_seed(seed), Some(rules), "{rules:?}");
            let code = seed::code(seed);
            assert_eq!(seed::parse(&code), Some(seed));
            assert!(code.replace('-', "").len() <= seed::MAX_CODE_LEN, "{code}");
        }
    }

    #[test]
    fn the_kinds_rule_names_kinds_in_enemy_order() {
        assert_eq!(Rules::default().kinds(), []);
        let rules = Rules {
            kinds: 0b10001,
            ..default()
        };
        assert_eq!(rules.kinds(), [EnemyKind::Ball, EnemyKind::Shooter]);
    }

    #[test]
    fn the_count_rule_scales_and_rounds_the_count() {
        let count = |count| Rules { count, ..default() };
        assert_eq!(
            (0..4).map(|c| count(c).count(1)).collect::<Vec<_>>(),
            [1, 2, 2, 3]
        );
        assert_eq!(count(1).count(3), 5, "4.5 rounds up");
        assert_eq!(Rules::default().count(7), 7);
    }

    #[test]
    fn the_speed_target_and_start_rules_read_their_values() {
        assert_eq!(Rules::default().speed(), 1.);
        assert_eq!(
            Rules {
                speed: 2,
                ..default()
            }
            .speed(),
            1.5
        );
        assert_eq!(Rules::default().target(), 0.75);
        assert_eq!(
            Rules {
                target: 3,
                ..default()
            }
            .target(),
            0.9
        );
        assert_eq!(Rules::default().countdown(), None);
        assert_eq!(
            Rules {
                countdown: 1,
                ..default()
            }
            .countdown(),
            Some(60)
        );
        assert_eq!(
            Rules {
                countdown: 5,
                ..default()
            }
            .countdown(),
            Some(300)
        );
        assert_eq!(Rules::default().level(), 1);
        assert_eq!(
            Rules {
                start: 9,
                ..default()
            }
            .level(),
            10
        );
    }

    #[test]
    fn the_speed_rule_runs_the_physics_clock_faster() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Time<Physics>>();
        world.insert_resource(Rules {
            speed: 3,
            ..default()
        });
        world.run_system_once(set_speed).unwrap();
        assert_eq!(world.resource::<Time<Physics>>().relative_speed(), 2.);
        world.insert_resource(Rules::default());
        world.run_system_once(set_speed).unwrap();
        assert_eq!(world.resource::<Time<Physics>>().relative_speed(), 1.);
    }

    #[test]
    fn harder_rules_pay_more_and_easier_ones_less() {
        assert_eq!(Rules::default().pay(1000), 1000);
        assert_eq!(
            Rules {
                speed: 2,
                ..default()
            }
            .pay(1000),
            1500
        );
        assert_eq!(
            Rules {
                target: 1,
                ..default()
            }
            .pay(1000),
            750
        );
        let hard = Rules {
            speed: 3,
            count: 3,
            target: 3,
            no_items: true,
            ..default()
        };
        assert_eq!(hard.pay(1000), 7500, "2 x 2 x 1.5 x 1.25");
        assert_eq!(Rules::default().pay(u128::MAX), u128::MAX);
        assert_eq!(hard.pay(u128::MAX / 10), u128::MAX, "it saturates");
        assert_eq!(
            Rules {
                kinds: 1,
                countdown: 2,
                start: 4,
                ..default()
            }
            .pay(1000),
            1000,
            "kinds, countdown and start level pay as the normal game"
        );
    }

    #[test]
    fn no_items_turns_the_items_off() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(Rules::default());
        assert!(world.run_system_once(items_on).unwrap());
        world.insert_resource(Rules {
            no_items: true,
            ..default()
        });
        assert!(!world.run_system_once(items_on).unwrap());
    }

    #[test]
    fn a_code_with_an_unknown_version_or_value_has_no_rules() {
        let field = 77;
        let with = |bits: u64| bits << FIELD_BITS | field;
        assert_eq!(Rules::of_seed(with(0b10)), None, "version 2");
        assert_eq!(Rules::of_seed(with(1 << 5)), None, "version 0 with rules");
        assert_eq!(Rules::of_seed(with(1 | 6 << 11)), None, "countdown 6");
        assert_eq!(Rules::of_seed(with(1 | 2 << 16)), None, "items 2");
        assert_eq!(Rules::of_seed(with(1 | 1 << 22)), None, "a spare bit");
        assert!(Rules::of_seed(with(1 | 5 << 11)).is_some(), "300 s");
    }
}
