use std::time::Duration;

use crate::logic::game::GameState;

pub fn check_game_over(game: &GameState) -> bool {
    game.total_lines >= 40
}

pub fn primary_stat(game: &GameState) -> String {
    let ms = game
        .play_start_time
        .map(|start| {
            let pause_adjust = game
                .pause_start
                .map(|ps| ps.elapsed())
                .unwrap_or(Duration::ZERO);
            start
                .elapsed()
                .saturating_sub(game.paused_accumulated + pause_adjust)
                .as_millis()
        })
        .unwrap_or(0);
    let mins = ms / 60000;
    let secs = (ms % 60000) / 1000;
    let millis = ms % 1000;
    format!("{:02}:{:02}.{:03}", mins, secs, millis)
}
