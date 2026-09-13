use crate::logic::game::GameState;

pub fn check_game_over(_game: &GameState) -> bool {
    false
}

pub fn primary_stat(game: &GameState) -> String {
    game.score.to_string()
}

pub fn calculate_garbage(
    lines_cleared: u32,
    is_tspin: bool,
    is_mini: bool,
    b2b: i32,
    combo: i32,
    perfect_clear: bool,
) -> u32 {
    let base = if is_tspin && !is_mini {
        match lines_cleared {
            1 => 2,
            2 => 4,
            3 => 6,
            _ => 0,
        }
    } else {
        match lines_cleared {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 4,
            _ => 0,
        }
    };

    let b2b_bonus = if b2b > 1 && lines_cleared > 0 && (lines_cleared == 4 || is_tspin) {
        1
    } else {
        0
    };

    let combo_bonus = if combo >= 2 {
        (combo / 2).min(5) as u32
    } else {
        0
    };

    let pc_bonus = if perfect_clear { 10 } else { 0 };

    base + b2b_bonus + combo_bonus + pc_bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_clears() {
        assert_eq!(calculate_garbage(0, false, false, 0, 0, false), 0);
        assert_eq!(calculate_garbage(1, false, false, 0, 0, false), 0);
        assert_eq!(calculate_garbage(2, false, false, 0, 0, false), 1);
        assert_eq!(calculate_garbage(3, false, false, 0, 0, false), 2);
        assert_eq!(calculate_garbage(4, false, false, 0, 0, false), 4);
    }

    #[test]
    fn tspin_clears() {
        assert_eq!(calculate_garbage(1, true, false, 0, 0, false), 2);
        assert_eq!(calculate_garbage(2, true, false, 0, 0, false), 4);
        assert_eq!(calculate_garbage(3, true, false, 0, 0, false), 6);
    }

    #[test]
    fn mini_tspin_uses_base() {
        assert_eq!(calculate_garbage(1, true, true, 0, 0, false), 0);
        assert_eq!(calculate_garbage(2, true, true, 0, 0, false), 1);
    }

    #[test]
    fn b2b_bonus() {
        assert_eq!(calculate_garbage(4, false, false, 2, 0, false), 5);
        assert_eq!(calculate_garbage(1, true, false, 2, 0, false), 3);
        assert_eq!(calculate_garbage(4, false, false, 1, 0, false), 4);
    }

    #[test]
    fn combo_bonus() {
        assert_eq!(calculate_garbage(1, false, false, 0, 1, false), 0);
        assert_eq!(calculate_garbage(1, false, false, 0, 2, false), 1);
        assert_eq!(calculate_garbage(1, false, false, 0, 3, false), 1);
        assert_eq!(calculate_garbage(1, false, false, 0, 4, false), 2);
        assert_eq!(calculate_garbage(1, false, false, 0, 10, false), 5);
        assert_eq!(calculate_garbage(1, false, false, 0, 20, false), 5);
    }

    #[test]
    fn perfect_clear() {
        assert_eq!(calculate_garbage(4, false, false, 0, 0, true), 14);
        assert_eq!(calculate_garbage(1, false, false, 0, 0, true), 10);
    }

    #[test]
    fn combined_b2b_combo_pc() {
        assert_eq!(calculate_garbage(4, false, false, 3, 6, true), 18);
    }
}
