use crate::logic::game::GameState;

const DURATION_SECS: f32 = 120.0;

pub fn check_game_over(game: &GameState) -> bool {
    game.play_start_time.map_or(false, |start| {
        let pause_adjust = game
            .pause_start
            .map(|ps| ps.elapsed())
            .unwrap_or(std::time::Duration::ZERO);
        start
            .elapsed()
            .saturating_sub(game.paused_accumulated + pause_adjust)
            .as_secs_f32()
            >= DURATION_SECS
    })
}

pub fn primary_stat(game: &GameState) -> String {
    game.score.to_string()
}
