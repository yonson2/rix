use fastrand::Rng;

/// Calculates the number of enemies for a given level with optional randomness
pub fn calculate_enemy_count(level: usize, rng: &mut Rng) -> usize {
    // Base count calculation based on level ranges
    let base_count = if level <= 10 {
        // Slow growth for first 10 levels
        1 + level / 3
    } else if level <= 25 {
        // Medium growth from 10-25
        4 + (level - 10) / 2
    } else {
        // Cap at 12 enemies maximum
        let late_game_count = 8 + (level - 25) / 5;
        if late_game_count > 12 {
            12
        } else {
            late_game_count
        }
    };

    // Calculate variance based on level (higher levels allow more variance)
    let variance = std::cmp::min(1, level / 5);

    // Apply random variance using fastrand
    if variance > 0 {
        // Generate random offset between -variance and +variance
        let offset = rng.i32(-(variance as i32)..=variance as i32);

        // Apply offset but ensure we don't go below 1 or above 15
        let final_count = base_count as i32 + offset;
        final_count.clamp(1, 15) as usize
    } else {
        base_count as usize
    }
}
