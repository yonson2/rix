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
