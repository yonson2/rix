use fastrand::Rng;

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
    use std::ops::RangeInclusive;

    use super::*;

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
