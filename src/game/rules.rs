//! Challenge rules ([120], D39): a challenge is a seed whose 24 high bits hold its rules,
//! so a code gives every player the same challenge (D23). Rules of 0 are the normal game:
//! a new run's 40-bit seed ([067]) and every code so far keep their game. A challenge code
//! has up to 13 characters.

use bevy::prelude::*;

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
