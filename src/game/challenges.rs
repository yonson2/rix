//! The listed challenges ([123], D39): each a fixed code (a field and its rules), so every
//! player gets the same game. Some open with games played ([114], [109] Q4); a shared or
//! typed code always plays.

use super::rules::Rules;

pub struct Challenge {
    pub name: &'static str,
    /// What its rules do, in one line.
    pub about: &'static str,
    pub rules: Rules,
    /// The seed under the rules.
    pub field: u64,
    /// The games played that open it ([114]); 0: open from the start.
    pub opens_at: u64,
}

impl Challenge {
    pub fn seed(&self) -> u64 {
        self.rules.seed(self.field)
    }
}

const NORMAL: Rules = Rules {
    kinds: 0,
    speed: 0,
    count: 0,
    countdown: 0,
    target: 0,
    no_items: false,
    start: 0,
};

/// In the order they open; the bits and indices are those of `Rules`.
pub const CHALLENGES: [Challenge; 10] = [
    Challenge {
        name: "60 SECONDS",
        about: "THE MOST POINTS IN ONE MINUTE",
        rules: Rules {
            countdown: 1,
            ..NORMAL
        },
        field: 0x44_0D67_B366,
        opens_at: 0,
    },
    Challenge {
        name: "BALL PIT",
        about: "BALLS ONLY, 2X ENEMIES",
        rules: Rules {
            kinds: 0b00001,
            count: 2,
            ..NORMAL
        },
        field: 0xC4_1651_DDAE,
        opens_at: 0,
    },
    Challenge {
        name: "BARE HANDS",
        about: "NO ITEMS",
        rules: Rules {
            no_items: true,
            ..NORMAL
        },
        field: 0x44_6840_FF07,
        opens_at: 0,
    },
    Challenge {
        name: "OVERDRIVE",
        about: "1.5X SPEED, MORE POINTS",
        rules: Rules { speed: 2, ..NORMAL },
        field: 0xD6_1B92_3E0A,
        opens_at: 10,
    },
    Challenge {
        name: "SWARM",
        about: "3X ENEMIES",
        rules: Rules { count: 3, ..NORMAL },
        field: 0xDF_E6B4_F832,
        opens_at: 15,
    },
    Challenge {
        name: "BULLET HELL",
        about: "SHOOTERS ONLY",
        rules: Rules {
            kinds: 0b10000,
            ..NORMAL
        },
        field: 0x61_09C4_7055,
        opens_at: 20,
    },
    Challenge {
        name: "SPLIT DECISION",
        about: "SPLITTERS ONLY",
        rules: Rules {
            kinds: 0b01000,
            ..NORMAL
        },
        field: 0x8F_8944_897C,
        opens_at: 30,
    },
    Challenge {
        name: "THREE MINUTES",
        about: "THE MOST POINTS IN THREE MINUTES",
        rules: Rules {
            countdown: 4,
            ..NORMAL
        },
        field: 0x57_550A_E889,
        opens_at: 40,
    },
    Challenge {
        name: "GREEDY",
        about: "90 % TO CLEAR A LEVEL",
        rules: Rules {
            target: 3,
            ..NORMAL
        },
        field: 0x0D_DA22_594F,
        opens_at: 60,
    },
    Challenge {
        name: "DEEP END",
        about: "START AT LEVEL 10",
        rules: Rules { start: 9, ..NORMAL },
        field: 0x22_28DF_A298,
        opens_at: 75,
    },
];

/// The name `seed` goes by: a listed challenge's, "CUSTOM CHALLENGE" for other rules, none
/// for the normal game.
pub fn name(seed: u64) -> Option<&'static str> {
    Rules::of_seed(seed).filter(|rules| rules.is_challenge())?;
    let listed = CHALLENGES.iter().find(|c| c.seed() == seed);
    Some(listed.map_or("CUSTOM CHALLENGE", |c| c.name))
}

/// The challenge that the fewest more games open ([113]).
pub fn next_by_games(games: u64) -> Option<&'static Challenge> {
    CHALLENGES.iter().find(|c| c.opens_at > games)
}

/// The notice for the challenges that playing game `new` opened, after `old` games.
pub fn notice(old: u64, new: u64) -> Option<String> {
    let names: Vec<_> = CHALLENGES
        .iter()
        .filter(|c| old < c.opens_at && c.opens_at <= new)
        .map(|c| c.name)
        .collect();
    (!names.is_empty())
        .then(|| format!("NEW CHALLENGE: {}\nPLAY IT IN CHALLENGES", names.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{enemies::rng::EnemyKind, rules::FIELD_BITS, seed};

    #[test]
    fn every_listed_code_parses_to_its_rules_and_name() {
        for c in &CHALLENGES {
            assert!(c.field < 1 << FIELD_BITS, "{}", c.name);
            let parsed = seed::parse(&seed::code(c.seed())).unwrap();
            assert_eq!(Rules::of_seed(parsed), Some(c.rules), "{}", c.name);
            assert_eq!(name(parsed), Some(c.name));
        }
        let rules = |name: &str| CHALLENGES.iter().find(|c| c.name == name).unwrap().rules;
        assert_eq!(rules("60 SECONDS").countdown(), Some(60));
        assert_eq!(rules("THREE MINUTES").countdown(), Some(180));
        assert_eq!(rules("BALL PIT").kinds(), [EnemyKind::Ball]);
        assert_eq!(rules("BALL PIT").count(4), 8);
        assert_eq!(rules("SWARM").count(4), 12);
        assert_eq!(rules("BULLET HELL").kinds(), [EnemyKind::Shooter]);
        assert_eq!(rules("SPLIT DECISION").kinds(), [EnemyKind::Splitter]);
        assert_eq!(rules("OVERDRIVE").speed(), 1.5);
        assert_eq!(rules("GREEDY").target(), 0.9);
        assert!(rules("BARE HANDS").no_items);
        assert_eq!(rules("DEEP END").level(), 10);
    }

    #[test]
    fn a_code_off_the_list_is_a_custom_challenge() {
        assert_eq!(name(12_345), None, "the normal game");
        let other = Rules { count: 3, ..NORMAL };
        assert_eq!(name(other.seed(5)), Some("CUSTOM CHALLENGE"));
        let swarm = &CHALLENGES[4];
        assert_eq!(
            name(swarm.rules.seed(swarm.field + 1)),
            Some("CUSTOM CHALLENGE")
        );
        assert_eq!(name(0b10 << FIELD_BITS), None, "an unknown version");
    }

    #[test]
    fn games_played_open_the_challenges_in_order() {
        let open: Vec<_> = CHALLENGES.iter().map(|c| c.opens_at).collect();
        assert!(open.is_sorted());
        assert_eq!(open.iter().filter(|&&at| at == 0).count(), 3);
        assert_eq!(next_by_games(0).map(|c| c.name), Some("OVERDRIVE"));
        assert_eq!(next_by_games(15).map(|c| c.name), Some("BULLET HELL"));
        assert!(next_by_games(75).is_none());
        assert_eq!(notice(9, 9), None);
        assert_eq!(
            notice(9, 10).as_deref(),
            Some("NEW CHALLENGE: OVERDRIVE\nPLAY IT IN CHALLENGES")
        );
    }

    #[test]
    fn no_challenge_opens_with_a_look() {
        use crate::game::skins;

        for c in CHALLENGES.iter().filter(|c| c.opens_at > 0) {
            let at = c.opens_at;
            assert!(
                skins::unlocked_by_games(at - 1, at).is_empty(),
                "{}",
                c.name
            );
        }
    }
}
