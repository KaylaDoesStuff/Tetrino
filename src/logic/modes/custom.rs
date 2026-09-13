use crate::logic::game::GameState;

pub fn check_game_over(_game: &GameState) -> bool {
    false
}

pub fn primary_stat(game: &GameState) -> String {
    game.score.to_string()
}
