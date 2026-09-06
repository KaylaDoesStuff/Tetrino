use std::collections::VecDeque;
use std::time::{Duration, Instant};

use egui::Color32;
use rand::seq::SliceRandom;
use rand::thread_rng;

use super::piece::Piece;
use super::settings::Settings;
use super::finesse;

pub struct GameState {
    pub grid: [[Option<Color32>; 10]; 23],
    pub bag_queue: VecDeque<i32>,
    pub active_piece: Option<Piece>,
    pub active_cells: [(i32, i32); 4],
    pub active_pos: (i32, i32),
    pub locking: bool,
    pub rotation_state: u8,
    pub level: i32,
    pub levels_counter: f32,
    pub last_tick_time: Instant,
    pub arr_ms: f32,
    pub das_ms: f32,
    pub dcd_ms: f32,
    pub sdf: f32,
    pub infinite_sdf: bool,
    pub das_direction: Option<i32>,
    pub das_charge: f32,
    pub arr_timer: f32,
    pub soft_dropping: bool,
    pub soft_drop_timer: f32,
    pub last_clear: u32,
    pub score: u64,
    pub drop_points: u32,
    pub combo: i32,
    pub b2b: i32,
    pub last_action_was_rotation: bool,
    pub last_action_used_kicks: bool,
    pub last_spin_piece: Option<Piece>,
    pub last_spin_mini: bool,
    pub hold_piece: Option<Piece>,
    pub hold_used: bool,
    pub total_lines: u32,
    pub was_soft_dropped: bool,
    pub finesse_input_count: u32,
    pub finesse_faults: u32,
    pub finesse_pieces_placed: u32,
    pub finesse_perfect_pieces: u32,
    pub total_keys_pressed: u32,
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub quads: u32,
    pub total_spins: u32,
    pub max_b2b: i32,
    pub max_combo: i32,
    pub play_start_time: Option<Instant>,
    pub pause_start: Option<Instant>,
    pub paused_accumulated: Duration,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            grid: [[None; 10]; 23],
            bag_queue: VecDeque::new(),
            active_piece: None,
            active_cells: [(0, 0); 4],
            active_pos: (0, 0),
            locking: false,
            rotation_state: 0,
            level: 1,
            levels_counter: 60.0,
            last_tick_time: Instant::now(),
            arr_ms: 1.0,
            das_ms: 100.0,
            dcd_ms: 50.0,
            sdf: 1.0,
            infinite_sdf: false,
            das_direction: None,
            das_charge: 0.0,
            arr_timer: 0.0,
            soft_dropping: false,
            soft_drop_timer: 0.0,
            last_clear: 0,
            score: 0,
            drop_points: 0,
            combo: 0,
            b2b: 0,
            last_action_was_rotation: false,
            last_action_used_kicks: false,
            last_spin_piece: None,
            last_spin_mini: false,
            hold_piece: None,
            hold_used: false,
            total_lines: 0,
            was_soft_dropped: false,
            finesse_input_count: 0,
            finesse_faults: 0,
            finesse_pieces_placed: 0,
            finesse_perfect_pieces: 0,
            total_keys_pressed: 0,
            singles: 0,
            doubles: 0,
            triples: 0,
            quads: 0,
            total_spins: 0,
            max_b2b: 0,
            max_combo: 0,
            play_start_time: None,
            pause_start: None,
            paused_accumulated: Duration::ZERO,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        self.arr_ms = settings.arr;
        self.das_ms = settings.das;
        self.dcd_ms = settings.dcd;
        self.sdf = settings.sdf;
        self.infinite_sdf = settings.infinite_sdf;
    }

    pub fn fill_bag(&mut self) {
        let mut bag = [1, 2, 3, 4, 5, 6, 7];
        bag.shuffle(&mut thread_rng());
        self.bag_queue.extend(bag.iter().copied());
    }

    pub fn spawn_random_piece(&mut self) {
        if self.bag_queue.len() < 7 {
            self.fill_bag();
        }

        let id = self.bag_queue.pop_front().unwrap();
        self.active_piece = Piece::from_id(id);

        if let Some(piece) = self.active_piece {
            self.active_cells = piece.spawn_cells();
            self.active_pos = (0, 3);
            self.locking = false;
            self.rotation_state = 0;
            self.das_direction = None;
            self.das_charge = 0.0;
            self.arr_timer = 0.0;
            self.soft_drop_timer = 0.0;
            self.was_soft_dropped = false;
            self.hold_used = false;
            self.drop_points = 0;
        }
    }

    pub fn hold_piece(&mut self) {
        if self.hold_used || self.active_piece.is_none() {
            return;
        }
        self.hold_used = true;
        let current = self.active_piece.take();
        match self.hold_piece {
            None => {
                self.hold_piece = current;
                self.was_soft_dropped = false;
                self.spawn_random_piece();
            }
            Some(held) => {
                self.hold_piece = current;
                self.active_piece = Some(held);
                self.active_cells = held.spawn_cells();
                self.active_pos = (0, 3);
                self.locking = false;
                self.rotation_state = 0;
                self.das_direction = None;
                self.das_charge = 0.0;
                self.arr_timer = 0.0;
                self.soft_drop_timer = 0.0;
                self.soft_dropping = false;
                self.was_soft_dropped = false;
                self.last_action_was_rotation = false;
            }
        }
    }

    pub fn try_move(&mut self, dc: i32) -> bool {
        if self.active_piece.is_none() {
            return false;
        }
        for &(dr, dc_cell) in &self.active_cells {
            let r = self.active_pos.0 + dr;
            let c = self.active_pos.1 + dc_cell + dc;
            if c < 0 || c >= 10 || r < 0 || r >= 23 {
                return false;
            }
            if self.grid[r as usize][c as usize].is_some() {
                return false;
            }
        }
        self.active_pos.1 += dc;
        self.last_action_was_rotation = false;
        true
    }

    pub fn try_move_down(&mut self) -> bool {
        if self.active_piece.is_none() {
            return false;
        }
        for &(dr, dc) in &self.active_cells {
            let r = self.active_pos.0 + dr + 1;
            let c = self.active_pos.1 + dc;
            if r >= 23 || self.grid[r as usize][c as usize].is_some() {
                return false;
            }
        }
        self.active_pos.0 += 1;
        self.last_action_was_rotation = false;
        true
    }

    pub fn hard_drop(&mut self) {
        if self.active_piece.is_none() {
            return;
        }
        let mut cells = 0u32;
        loop {
            let mut can_move = true;
            for &(dr, dc) in &self.active_cells {
                let r = self.active_pos.0 + dr + 1;
                let c = self.active_pos.1 + dc;
                if r >= 23 || self.grid[r as usize][c as usize].is_some() {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                self.active_pos.0 += 1;
                cells += 1;
            } else {
                break;
            }
        }
        self.drop_points += cells * 2;
        self.lock_piece();
    }

    pub fn apply_dcd(&mut self) {
        self.das_charge -= self.dcd_ms;
        if self.das_charge < 0.0 {
            self.das_charge = 0.0;
        }
    }

    pub fn cells_valid_at(&self, cells: &[(i32, i32); 4], pos: (i32, i32)) -> bool {
        for &(dr, dc) in cells {
            let r = pos.0 + dr;
            let c = pos.1 + dc;
            if r < 0 || r >= 23 || c < 0 || c >= 10 {
                return false;
            }
            if self.grid[r as usize][c as usize].is_some() {
                return false;
            }
        }
        true
    }

    pub fn is_game_over(&self) -> bool {
        if let Some(_piece) = self.active_piece {
            !self.cells_valid_at(&self.active_cells, self.active_pos)
        } else {
            false
        }
    }

    fn corner_filled(&self, r: i32, c: i32) -> bool {
        if r < 0 || r >= 23 || c < 0 || c >= 10 {
            true
        } else {
            self.grid[r as usize][c as usize].is_some()
        }
    }

    pub fn check_tspin(&self) -> (bool, bool) {
        let piece = match self.active_piece {
            Some(p) => p,
            None => return (false, false),
        };
        if piece != Piece::T || !self.last_action_was_rotation {
            return (false, false);
        }

        let r0 = self.active_pos.0;
        let c0 = self.active_pos.1;

        let corners = [
            self.corner_filled(r0, c0),
            self.corner_filled(r0, c0 + 2),
            self.corner_filled(r0 + 2, c0),
            self.corner_filled(r0 + 2, c0 + 2),
        ];
        let count = corners.iter().filter(|&&f| f).count();
        if count < 3 {
            return (false, false);
        }

        let front = match self.rotation_state {
            0 => [self.corner_filled(r0 + 2, c0), self.corner_filled(r0 + 2, c0 + 2)],
            1 => [self.corner_filled(r0, c0 + 2), self.corner_filled(r0 + 2, c0 + 2)],
            2 => [self.corner_filled(r0, c0), self.corner_filled(r0, c0 + 2)],
            _ => [self.corner_filled(r0, c0), self.corner_filled(r0 + 2, c0)],
        };
        let front_count = front.iter().filter(|&&f| f).count();

        (true, front_count < 2)
    }

    pub fn check_spin(&self) -> Option<(Piece, bool)> {
        let piece = self.active_piece?;
        if !self.last_action_was_rotation {
            return None;
        }
        if piece == Piece::T {
            let (is_tspin, is_mini) = self.check_tspin();
            if is_tspin {
                return Some((Piece::T, is_mini));
            }
            return None;
        }
        let immobile = !self.cells_valid_at(&self.active_cells, (self.active_pos.0, self.active_pos.1 - 1))
            && !self.cells_valid_at(&self.active_cells, (self.active_pos.0, self.active_pos.1 + 1))
            && !self.cells_valid_at(&self.active_cells, (self.active_pos.0 - 1, self.active_pos.1));
        if immobile {
            Some((piece, true))
        } else {
            None
        }
    }

    pub fn lock_piece(&mut self) {
        let locked_piece = self.active_piece;
        let locked_rotation = self.rotation_state;

        let (spin_piece, mut spin_mini) = match self.check_spin() {
            Some((p, mini)) => (Some(p), mini),
            None => (None, false),
        };
        let is_tspin = spin_piece == Some(Piece::T);
        let is_mini = spin_mini && is_tspin;

        if spin_piece.is_some() {
            self.total_spins += 1;
        }

        if let Some(piece) = self.active_piece {
            let color = Some(piece.color());
            for &(dr, dc) in &self.active_cells {
                let r = (self.active_pos.0 + dr) as usize;
                let c = (self.active_pos.1 + dc) as usize;
                if r < 23 && c < 10 {
                    self.grid[r][c] = color;
                }
            }
        }
        self.active_piece = None;

        let mut cleared = 0u32;
        let mut row = 0;
        while row < 23 {
            if (0..10).all(|c| self.grid[row][c].is_some()) {
                for r in (1..=row).rev() {
                    self.grid[r] = self.grid[r - 1];
                }
                self.grid[0] = [None; 10];
                cleared += 1;
            } else {
                row += 1;
            }
        }

        let perfect_clear = self.grid.iter().all(|row| row.iter().all(|c| c.is_none()));

        match cleared {
            1 => self.singles += 1,
            2 => self.doubles += 1,
            3 => self.triples += 1,
            4 => self.quads += 1,
            _ => {}
        }

        if spin_piece.is_some() && spin_piece != Some(Piece::T) && cleared >= 3 {
            spin_mini = false;
        }

        let base_score: u64 = if is_tspin {
            match cleared {
                0 => 400,
                1 => 800,
                2 => 1200,
                3 => 1600,
                _ => 1600,
            }
        } else if is_mini {
            match cleared {
                0 => 100,
                1 => 200,
                2 => 400,
                _ => 400,
            }
        } else {
            match cleared {
                0 => 0,
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800,
                _ => 0,
            }
        };

        let is_special = spin_piece.is_some() || cleared == 4;
        if cleared > 0 {
            self.combo += 1;
        } else {
            self.combo = 0;
        }
        self.max_combo = self.max_combo.max(self.combo);

        if is_special && cleared > 0 {
            self.b2b += 1;
        } else if cleared > 0 && !is_special {
            self.b2b = 0;
        }
        self.max_b2b = self.max_b2b.max(self.b2b);

        let b2b_mult: f64 = if self.b2b > 1 { 1.5 } else { 1.0 };
        let mut earned = (base_score as f64 * self.level as f64 * b2b_mult) as u64;

        earned += (self.combo * 50 * self.level) as u64;

        if perfect_clear {
            earned += 3500;
        }

        earned += self.drop_points as u64;
        self.score += earned;
        self.total_lines += cleared;
        self.level = 1 + (self.total_lines / 10) as i32;
        self.last_clear = cleared;
        self.last_spin_piece = spin_piece;
        self.last_spin_mini = spin_mini;
        self.last_action_was_rotation = false;

        if let Some(piece) = locked_piece {
            self.finesse_pieces_placed += 1;

            if self.was_soft_dropped {
            self.finesse_perfect_pieces += 1;
        } else {
            let col = self.active_pos.1 as usize;

            if let Some(optimal) =
                finesse::lookup_finesse(piece, col, locked_rotation)
            {
                if self.finesse_input_count == optimal as u32 {
                    self.finesse_perfect_pieces += 1;
                } else if self.finesse_input_count > optimal as u32 {
                    self.finesse_faults +=
                    self.finesse_input_count - optimal as u32;
                }
            } else {
                self.finesse_perfect_pieces += 1;
            }
        }
    }
    self.finesse_input_count = 0;

        self.spawn_random_piece();
    }

    pub fn compute_ghost_pos(&self) -> Option<(i32, i32)> {
        if self.active_piece.is_none() {
            return None;
        }
        let mut gp = self.active_pos;
        loop {
            let mut can_move = true;
            for &(dr, dc) in &self.active_cells {
                let new_r = gp.0 + 1 + dr;
                let c = gp.1 + dc;
                if new_r >= 23 || self.grid[new_r as usize][c as usize].is_some() {
                    can_move = false;
                    break;
                }
            }
            if can_move {
                gp.0 += 1;
            } else {
                break;
            }
        }
        Some(gp)
    }

    pub fn tick(&mut self, elapsed: f32) {
        let elapsed_ms = elapsed * 1000.0;

        if let Some(dir) = self.das_direction {
            self.das_charge += elapsed_ms;
            if self.das_charge >= self.das_ms {
                self.arr_timer += elapsed_ms;
                if self.arr_ms == 0.0 {
                    while self.try_move(dir) {}
                    self.arr_timer = 0.0;
                } else {
                    while self.arr_timer >= self.arr_ms {
                        self.try_move(dir);
                        self.arr_timer -= self.arr_ms;
                    }
                }
            }
        } else {
            self.arr_timer = 0.0;
        }

        if self.soft_dropping && self.active_piece.is_some() {
            self.was_soft_dropped = true;
            if self.infinite_sdf {
                while self.try_move_down() {
                    self.drop_points += 1;
                }
                self.soft_dropping = false;
            } else {
                self.soft_drop_timer += elapsed;
                let interval = 1.0 / self.sdf;
                while self.soft_drop_timer >= interval {
                    self.soft_drop_timer -= interval;
                    if !self.try_move_down() {
                        self.soft_dropping = false;
                        break;
                    } else {
                        self.drop_points += 1;
                    }
                }
            }
        }

        self.levels_counter -= elapsed * 60.0;
        if self.levels_counter <= 0.0 {
            self.levels_counter = 60.0 / self.level as f32;

            if self.locking {
                self.lock_piece();
            } else if self.active_piece.is_some() {
                let mut can_move = true;
                for &(dr, dc) in &self.active_cells {
                    let new_r = self.active_pos.0 + 1 + dr;
                    let c = self.active_pos.1 + dc;
                    if new_r >= 23 || self.grid[new_r as usize][c as usize].is_some() {
                        can_move = false;
                        break;
                    }
                }

                if can_move {
                    self.active_pos.0 += 1;
                    self.last_action_was_rotation = false;
                } else {
                    self.locking = true;
                }
            }
        }
    }
}
