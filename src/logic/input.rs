use std::sync::Arc;

use winit::event::ElementState;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{Fullscreen, Window};

use super::game::GameState;
use super::graphics::GpuState;
use super::piece::Piece;
use super::settings::Settings;

pub fn handle_key_event(
    game: &mut GameState,
    window: Option<&Arc<Window>>,
    gpu: &mut Option<GpuState>,
    vsync: &mut bool,
    key: &Key,
    state: ElementState,
    event_loop: &ActiveEventLoop,
    settings: &Settings,
) {
    match state {
        ElementState::Pressed => {
            if settings.key_matches(key, "move_left") {
                game.finesse_input_count += 1;
                game.total_keys_pressed += 1;
                let dir = game.das_direction;
                if dir != Some(-1) {
                    game.try_move(-1);
                    game.das_direction = Some(-1);
                    game.das_charge = 0.0;
                    game.arr_timer = 0.0;
                }
            } else if settings.key_matches(key, "move_right") {
                game.finesse_input_count += 1;
                game.total_keys_pressed += 1;
                let dir = game.das_direction;
                if dir != Some(1) {
                    game.try_move(1);
                    game.das_direction = Some(1);
                    game.das_charge = 0.0;
                    game.arr_timer = 0.0;
                }
            } else if settings.key_matches(key, "soft_drop") {
                game.soft_dropping = true;
                game.soft_drop_timer = 0.0;
            } else if settings.key_matches(key, "hard_drop") {
                game.total_keys_pressed += 1;
                game.hard_drop();
                game.apply_dcd();
            } else if settings.key_matches(key, "hold") {
                game.finesse_input_count = 0;
                game.hold_piece();
            } else if settings.key_matches(key, "rotate_ccw") {
                game.finesse_input_count += 1;
                game.total_keys_pressed += 1;
                if let Some(piece) = game.active_piece {
                    let old_state = game.rotation_state;
                    let new_state = (old_state + 3) % 4;
                    let new_cells = Piece::rotate_ccw(&game.active_cells, piece);
                    let kicks = Piece::get_kick_offsets(piece, old_state, new_state);
                    for (i, &(dx, dy)) in kicks.iter().enumerate() {
                        let test_pos = (game.active_pos.0 - dy, game.active_pos.1 + dx);
                        if game.cells_valid_at(&new_cells, test_pos) {
                            game.active_cells = new_cells;
                            game.active_pos = test_pos;
                            game.rotation_state = new_state;
                            game.apply_dcd();
                            game.last_action_was_rotation = true;
                            game.last_action_used_kicks = i > 0;
                            break;
                        }
                    }
                }
            } else if settings.key_matches(key, "rotate_cw") {
                game.finesse_input_count += 1;
                game.total_keys_pressed += 1;
                if let Some(piece) = game.active_piece {
                    let old_state = game.rotation_state;
                    let new_state = (old_state + 1) % 4;
                    let new_cells = Piece::rotate_cw(&game.active_cells, piece);
                    let kicks = Piece::get_kick_offsets(piece, old_state, new_state);
                    for (i, &(dx, dy)) in kicks.iter().enumerate() {
                        let test_pos = (game.active_pos.0 - dy, game.active_pos.1 + dx);
                        if game.cells_valid_at(&new_cells, test_pos) {
                            game.active_cells = new_cells;
                            game.active_pos = test_pos;
                            game.rotation_state = new_state;
                            game.apply_dcd();
                            game.last_action_was_rotation = true;
                            game.last_action_used_kicks = i > 0;
                            break;
                        }
                    }
                }
            } else if settings.key_matches(key, "rotate_180") {
                game.finesse_input_count += 1;
                game.total_keys_pressed += 1;
                if let Some(piece) = game.active_piece {
                    let new_cells = Piece::flip_180(&game.active_cells, piece);
                    let mut valid = true;
                    for &(dr, dc) in &new_cells {
                        let r = game.active_pos.0 + dr;
                        let c = game.active_pos.1 + dc;
                        if r < 0 || r >= 23 || c < 0 || c >= 10 {
                            valid = false;
                            break;
                        }
                        if game.grid[r as usize][c as usize].is_some() {
                            valid = false;
                            break;
                        }
                    }
                    if valid {
                        game.active_cells = new_cells;
                        game.rotation_state = (game.rotation_state + 2) % 4;
                        game.apply_dcd();
                        game.last_action_was_rotation = true;
                        game.last_action_used_kicks = false;
                    }
                }
            } else if let Key::Named(winit::keyboard::NamedKey::F11) = key {
                if let Some(window) = window {
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                    } else {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                }
            } else if let Key::Named(winit::keyboard::NamedKey::F5) = key {
                *vsync = !*vsync;
                if let Some(gpu) = gpu {
                    let mode = if *vsync {
                        wgpu::PresentMode::Fifo
                    } else {
                        wgpu::PresentMode::Immediate
                    };
                    gpu.set_present_mode(mode);
                }
            } else if let Key::Named(winit::keyboard::NamedKey::F1) = key {
                game.spawn_random_piece();
            } else if let Key::Named(winit::keyboard::NamedKey::Escape) = key {
                event_loop.exit();
            }
        }
        ElementState::Released => {
            if settings.key_matches(key, "move_left") {
                if game.das_direction == Some(-1) {
                    game.das_direction = None;
                }
            } else if settings.key_matches(key, "move_right") {
                if game.das_direction == Some(1) {
                    game.das_direction = None;
                }
            } else if settings.key_matches(key, "soft_drop") {
                game.soft_dropping = false;
                game.soft_drop_timer = 0.0;
            }
        }
    }
}
