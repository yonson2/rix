use std::ops::RangeInclusive;

use fastrand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Ball,
    Square,
    Arrow,
}

/// A kind can spawn on `levels` with this `weight`; a kind can have one row per band.
pub struct SpawnRule {
    pub kind: EnemyKind,
    pub levels: RangeInclusive<usize>,
    pub weight: u64,
}

/// Every kind from level 1 with equal weights: the pick before [032].
pub const SPAWN_TABLE: &[SpawnRule] = &[
    SpawnRule {
        kind: EnemyKind::Ball,
        levels: 1..=usize::MAX,
        weight: 1,
    },
    SpawnRule {
        kind: EnemyKind::Square,
        levels: 1..=usize::MAX,
        weight: 1,
    },
    SpawnRule {
        kind: EnemyKind::Arrow,
        levels: 1..=usize::MAX,
        weight: 1,
    },
];

/// A weighted pick of the rules for `level`. It draws `u64`, not `usize`, so a seed picks
/// the same on 32-bit targets (wasm); on 64-bit it draws as `Rng::choice` did.
pub fn pick_kind(table: &[SpawnRule], level: usize, rng: &mut Rng) -> Option<EnemyKind> {
    let rules: Vec<&SpawnRule> = table.iter().filter(|r| r.levels.contains(&level)).collect();
    let total: u64 = rules.iter().map(|r| r.weight).sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng.u64(0..total);
    rules.into_iter().find_map(|r| {
        if roll < r.weight {
            Some(r.kind)
        } else {
            roll -= r.weight;
            None
        }
    })
}

/// Calculates the number of enemies for a given level with optional randomness
pub fn calculate_enemy_count(level: usize, rng: &mut Rng) -> usize {
    let base_count = if level == 0 {
        1
    } else if level <= 10 {
        1 + level / 3
    } else if level <= 25 {
        4 + (level - 10) / 2
    } else {
        let late_game_count = 8 + (level - 25) / 5;
        if late_game_count > 12 {
            12
        } else {
            late_game_count
        }
    };

    if level < 5 {
        return base_count;
    }

    let variance = std::cmp::min(2, level / 5);

    if variance > 0 {
        let offset = rng.i32(-(variance as i32)..=variance as i32);

        let final_count = base_count as i32 + offset;
        final_count.clamp(1, 15) as usize
    } else {
        base_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use EnemyKind::*;

    /// The count, then one pick per enemy, as `spawn_enemies` draws them (less the
    /// draws of each spawn).
    fn level_kinds(table: &[SpawnRule], level: usize, seed: u64) -> Vec<EnemyKind> {
        let mut rng = Rng::with_seed(seed);
        let count = calculate_enemy_count(level, &mut rng);
        (0..count)
            .map(|_| pick_kind(table, level, &mut rng).unwrap())
            .collect()
    }

    #[test]
    fn the_table_gives_these_kinds_at_levels_1_10_and_30() {
        assert_eq!(level_kinds(SPAWN_TABLE, 1, 1), [Square]);
        assert_eq!(
            level_kinds(SPAWN_TABLE, 10, 1),
            [Square, Ball, Arrow, Ball, Arrow]
        );
        assert_eq!(
            level_kinds(SPAWN_TABLE, 30, 1),
            [
                Square, Ball, Arrow, Ball, Arrow, Arrow, Arrow, Square, Ball, Ball
            ]
        );
    }

    #[test]
    fn equal_weights_pick_as_choice_did() {
        let kinds = [Ball, Square, Arrow];
        let mut a = Rng::with_seed(9);
        let mut b = Rng::with_seed(9);
        for _ in 0..100 {
            assert_eq!(pick_kind(SPAWN_TABLE, 1, &mut a), b.choice(kinds));
        }
    }

    #[test]
    fn a_kind_spawns_only_in_its_band_and_by_weight() {
        let table = [
            SpawnRule {
                kind: Ball,
                levels: 1..=usize::MAX,
                weight: 1,
            },
            SpawnRule {
                kind: Arrow,
                levels: 5..=9,
                weight: 3,
            },
        ];
        let picks = |level| {
            let mut rng = Rng::with_seed(3);
            (0..4000)
                .map(|_| pick_kind(&table, level, &mut rng).unwrap())
                .filter(|k| *k == Arrow)
                .count()
        };
        assert_eq!((picks(4), picks(10)), (0, 0));
        assert!((2850..3150).contains(&picks(5)), "{}", picks(5));
        assert_eq!(pick_kind(&[], 1, &mut Rng::with_seed(1)), None);
    }

    /// Every count seen for `level` over many seeds.
    fn seen_range(level: usize) -> RangeInclusive<usize> {
        let counts: Vec<usize> = (0..500)
            .map(|seed| calculate_enemy_count(level, &mut Rng::with_seed(seed)))
            .collect();
        *counts.iter().min().unwrap()..=*counts.iter().max().unwrap()
    }

    #[test]
    fn levels_below_5_are_fixed() {
        let counts: Vec<usize> = (0..5)
            .map(|level| calculate_enemy_count(level, &mut Rng::new()))
            .collect();
        assert_eq!(counts, [1, 1, 1, 2, 2]);
    }

    #[test]
    fn variance_grows_to_two() {
        assert_eq!(seen_range(5), 1..=3);
        assert_eq!(seen_range(10), 2..=6);
        assert_eq!(seen_range(11), 2..=6);
    }

    #[test]
    fn quirk_count_drops_after_level_25() {
        assert_eq!(seen_range(25), 9..=13);
        assert_eq!(seen_range(26), 6..=10);
    }

    #[test]
    fn late_game_caps_at_12_plus_variance() {
        assert_eq!(seen_range(45), 10..=14);
        assert_eq!(seen_range(1000), 10..=14);
    }

    #[test]
    fn same_seed_same_counts() {
        let run = |seed| {
            let mut rng = Rng::with_seed(seed);
            (1..40)
                .map(|level| calculate_enemy_count(level, &mut rng))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(1), run(1));
    }
}
