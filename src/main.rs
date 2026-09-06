#![windows_subsystem = "windows"]

mod logic;

use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use egui::Pos2;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use logic::game::GameState;
use logic::graphics::{self, GpuState};
use logic::input;
use logic::menu::{self, MenuAction};
use logic::settings::{self, Settings, SettingsAction, SettingsUiState};

#[derive(Clone, Copy, PartialEq)]
enum GameMode {
    Sprint40L,
    Blitz,
    Custom,
}

enum AppScreen {
    Menu,
    ModeSelect,
    Settings,
    Countdown { remaining: f32 },
    Playing,
    Paused,
    GameOver { remaining: f32 },
    EndScreen,
}

struct App {
    window: Option<Arc<Window>>,
    egui_ctx: egui::Context,
    egui_winit: Option<egui_winit::State>,
    gpu: Option<GpuState>,
    game: GameState,
    vsync: bool,
    frame_times: VecDeque<Instant>,
    frame_count: u64,
    displayed_fps: usize,
    scale: f32,
    screen: AppScreen,
    game_mode: GameMode,
    mouse_pos: Option<Pos2>,
    settings: Settings,
    settings_ui: SettingsUiState,
    prev_settings: Settings,
    needs_settings_apply: bool,
    background_texture: Option<egui::TextureHandle>,
    last_frame_start: Instant,
    end_stats_timer: String,
    end_stats_pps: f32,
    end_stats_kpp: f32,
    end_stats_score: u64,
    end_stats_pieces: u32,
    end_stats_lines: u32,
    end_stats_level: i32,
    end_stats_singles: u32,
    end_stats_doubles: u32,
    end_stats_triples: u32,
    end_stats_quads: u32,
    end_stats_spins: u32,
    end_stats_max_b2b: i32,
    end_stats_max_combo: i32,
    end_stats_finesse_faults: u32,
    end_stats_finesse_pct: f32,
    end_stats_total_keys: u32,
}

impl App {
    fn new() -> Self {
        let settings = Settings::load();
        let vsync = settings.vsync;
        let mut app = Self {
            window: None,
            egui_ctx: egui::Context::default(),
            egui_winit: None,
            gpu: None,
            game: GameState::new(),
            vsync,
            frame_times: VecDeque::new(),
            frame_count: 0,
            displayed_fps: 0,
            scale: 1.0,
            screen: AppScreen::Menu,
            game_mode: GameMode::Custom,
            mouse_pos: None,
            prev_settings: settings.clone(),
            settings,
            settings_ui: SettingsUiState::default(),
            needs_settings_apply: false,
            background_texture: None,
            last_frame_start: Instant::now(),
            end_stats_timer: String::new(),
            end_stats_pps: 0.0,
            end_stats_kpp: 0.0,
            end_stats_score: 0,
            end_stats_pieces: 0,
            end_stats_lines: 0,
            end_stats_level: 0,
            end_stats_singles: 0,
            end_stats_doubles: 0,
            end_stats_triples: 0,
            end_stats_quads: 0,
            end_stats_spins: 0,
            end_stats_max_b2b: 0,
            end_stats_max_combo: 0,
            end_stats_finesse_faults: 0,
            end_stats_finesse_pct: 0.0,
            end_stats_total_keys: 0,
        };
        app.load_background_texture();
        app
    }

    fn reset_end_stats(&mut self) {
        self.end_stats_timer = String::new();
        self.end_stats_pps = 0.0;
        self.end_stats_kpp = 0.0;
        self.end_stats_score = 0;
        self.end_stats_pieces = 0;
        self.end_stats_lines = 0;
        self.end_stats_level = 0;
        self.end_stats_singles = 0;
        self.end_stats_doubles = 0;
        self.end_stats_triples = 0;
        self.end_stats_quads = 0;
        self.end_stats_spins = 0;
        self.end_stats_max_b2b = 0;
        self.end_stats_max_combo = 0;
        self.end_stats_finesse_faults = 0;
        self.end_stats_finesse_pct = 0.0;
        self.end_stats_total_keys = 0;
    }

    fn apply_settings(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            if self.settings.vsync != self.prev_settings.vsync {
                let mode = if self.settings.vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Immediate
                };
                gpu.set_present_mode(mode);
                self.vsync = self.settings.vsync;
            }
        }

        if self.settings.background_path != self.prev_settings.background_path {
            self.load_background_texture();
        }

        if let Some(window) = &self.window {
            if self.settings.fullscreen != self.prev_settings.fullscreen {
                if self.settings.fullscreen {
                    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                } else {
                    window.set_fullscreen(None);
                }
            }

            if self.settings.resolution != self.prev_settings.resolution {
                let (w, h) = self.settings.resolution;
                match window.request_inner_size(winit::dpi::LogicalSize::new(w, h)) {
                    Some(physical) => {
                        if let Some(gpu) = &mut self.gpu {
                            gpu.resize(physical.width, physical.height);
                        }
                    }
                    None => {} // Wayland: Resized event will handle it
                }
            }
        }

        self.prev_settings = self.settings.clone();
    }

    fn load_background_texture(&mut self) {
        match &self.settings.background_path {
            Some(path) => {
                match image::open(path) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            rgba.as_raw(),
                        );
                        let handle = self.egui_ctx.load_texture(
                            "background",
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        self.background_texture = Some(handle);
                    }
                    Err(_) => {
                        self.background_texture = None;
                    }
                }
            }
            None => {
                self.background_texture = None;
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (w, h) = self.settings.resolution;
        let (window, gpu, egui_winit) =
            graphics::init_graphics(event_loop, &self.egui_ctx, self.vsync, w, h);
        self.window = Some(window);
        self.gpu = Some(gpu);
        self.egui_winit = Some(egui_winit);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        if let (Some(window), Some(egui_winit)) = (&self.window, &mut self.egui_winit) {
            let _ = egui_winit.on_window_event(window, &event);
        }

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = &self.window {
                    let sf = window.scale_factor() as f32;
                    self.mouse_pos = Some(Pos2::new(
                        position.x as f32 / sf,
                        position.y as f32 / sf,
                    ));
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let AppScreen::Menu = &self.screen {
                    if let Some(pos) = self.mouse_pos {
                        let (surf_w, surf_h) = if let Some(gpu) = &self.gpu {
                            (gpu.surface_config.width as f32, gpu.surface_config.height as f32)
                        } else {
                            return;
                        };
                        let sf = self.window.as_ref().map(|w| w.scale_factor() as f32).unwrap_or(1.0);
                        let buttons = menu::build_buttons(surf_w, surf_h, sf);
                        match menu::handle_menu_click(pos, &buttons) {
                            MenuAction::Play => {
                                self.screen = AppScreen::ModeSelect;
                            }
                            MenuAction::Settings => {
                                self.prev_settings = self.settings.clone();
                                self.screen = AppScreen::Settings;
                            }
                            MenuAction::Exit => event_loop.exit(),
                            MenuAction::None => {}
                        }
                    }
                } else if let AppScreen::Paused = &self.screen {
                    if let Some(pos) = self.mouse_pos {
                        let (surf_w, surf_h) = if let Some(gpu) = &self.gpu {
                            (gpu.surface_config.width as f32, gpu.surface_config.height as f32)
                        } else {
                            return;
                        };
                        let sf = self.window.as_ref().map(|w| w.scale_factor() as f32).unwrap_or(1.0);
                        let buttons = graphics::build_pause_buttons(surf_w, surf_h, sf);
                        match graphics::handle_pause_click(pos, &buttons) {
                            graphics::PauseAction::Continue => {
                                if let Some(start) = self.game.pause_start.take() {
                                    self.game.paused_accumulated += start.elapsed();
                                }
                                self.screen = AppScreen::Playing;
                            }
                            graphics::PauseAction::Retry => {
                                self.game.reset();
                                self.game.apply_settings(&self.settings);
                                self.game.fill_bag();
                                self.game.fill_bag();
                                self.reset_end_stats();
                                self.screen = AppScreen::Countdown { remaining: 3.0 };
                            }
                            graphics::PauseAction::ExitToMenu => {
                                self.screen = AppScreen::Menu;
                            }
                            graphics::PauseAction::None => {}
                        }
                    }
                } else if let AppScreen::EndScreen = &self.screen {
                    if let Some(pos) = self.mouse_pos {
                        let (surf_w, surf_h) = if let Some(gpu) = &self.gpu {
                            (gpu.surface_config.width as f32, gpu.surface_config.height as f32)
                        } else {
                            return;
                        };
                        let sf = self.window.as_ref().map(|w| w.scale_factor() as f32).unwrap_or(1.0);
                        let exit_button = graphics::build_end_screen_exit_button(surf_w, surf_h, sf);
                        if graphics::handle_end_screen_click(pos, &exit_button) {
                            self.screen = AppScreen::Menu;
                        }
                    }
                } else if let AppScreen::ModeSelect = &self.screen {
                    if let Some(pos) = self.mouse_pos {
                        let (surf_w, surf_h) = if let Some(gpu) = &self.gpu {
                            (gpu.surface_config.width as f32, gpu.surface_config.height as f32)
                        } else {
                            return;
                        };
                        let sf = self.window.as_ref().map(|w| w.scale_factor() as f32).unwrap_or(1.0);
                        let buttons = menu::build_mode_buttons(surf_w, surf_h, sf);
                        match menu::handle_mode_click(pos, &buttons) {
                            menu::ModeAction::Sprint40L => {
                                self.game_mode = GameMode::Sprint40L;
                                self.game.reset();
                                self.game.apply_settings(&self.settings);
                                self.game.fill_bag();
                                self.game.fill_bag();
                                self.reset_end_stats();
                                self.screen = AppScreen::Countdown { remaining: 3.0 };
                            }
                            menu::ModeAction::Blitz => {
                                self.game_mode = GameMode::Blitz;
                                self.game.reset();
                                self.game.apply_settings(&self.settings);
                                self.game.fill_bag();
                                self.game.fill_bag();
                                self.reset_end_stats();
                                self.screen = AppScreen::Countdown { remaining: 3.0 };
                            }
                            menu::ModeAction::Custom => {
                                self.game_mode = GameMode::Custom;
                                self.game.reset();
                                self.game.apply_settings(&self.settings);
                                self.game.fill_bag();
                                self.game.fill_bag();
                                self.reset_end_stats();
                                self.screen = AppScreen::Countdown { remaining: 3.0 };
                            }
                            menu::ModeAction::None => {}
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state,
                        ..
                    },
                ..
            } => {
                if let (Key::Named(NamedKey::F11), ElementState::Pressed) =
                    (&logical_key, state)
                {
                    if let Some(window) = &self.window {
                        if window.fullscreen().is_some() {
                            window.set_fullscreen(None);
                        } else {
                            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        }
                    }
                }
                match &mut self.screen {
                    AppScreen::Menu => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            event_loop.exit();
                        }
                        if let (Key::Named(NamedKey::Enter), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.screen = AppScreen::ModeSelect;
                        }
                    }
                    AppScreen::ModeSelect => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.screen = AppScreen::Menu;
                        }
                    }
                    AppScreen::Settings => {
                        if settings::handle_settings_key(&logical_key, &mut self.settings_ui, &mut self.settings) {
                            return;
                        }
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.settings.save();
                            self.apply_settings();
                            self.screen = AppScreen::Menu;
                        }
                    }
                    AppScreen::Countdown { .. } => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.screen = AppScreen::Menu;
                        }
                    }
                    AppScreen::Playing => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.screen = AppScreen::Paused;
                            self.game.pause_start = Some(Instant::now());
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                            return;
                        }
                        input::handle_key_event(
                            &mut self.game,
                            self.window.as_ref(),
                            &mut self.gpu,
                            &mut self.vsync,
                            &logical_key,
                            *state,
                            event_loop,
                            &self.settings,
                        );
                    }
                    AppScreen::Paused => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            if let Some(start) = self.game.pause_start.take() {
                                self.game.paused_accumulated += start.elapsed();
                            }
                            self.screen = AppScreen::Playing;
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                    }
                    AppScreen::GameOver { .. } => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.screen = AppScreen::Menu;
                        }
                    }
                    AppScreen::EndScreen => {
                        if let (Key::Named(NamedKey::Escape), ElementState::Pressed) =
                            (&logical_key, state)
                        {
                            self.screen = AppScreen::Menu;
                        }
                    }
                }
            }

            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                if let Some(cap) = self.settings.fps_cap {
                    let target = Duration::from_secs_f64(1.0 / cap as f64);
                    let elapsed = now.duration_since(self.last_frame_start);
                    if elapsed < target {
                        thread::sleep(target - elapsed);
                    }
                }
                self.last_frame_start = Instant::now();
                self.frame_times.push_back(self.last_frame_start);
                while let Some(front) = self.frame_times.front() {
                    if self.last_frame_start.duration_since(*front).as_secs() > 1 {
                        self.frame_times.pop_front();
                    } else {
                        break;
                    }
                }
                self.frame_count += 1;
                if self.frame_count % 60 == 0 {
                    self.displayed_fps = self.frame_times.len();
                }

                if let (
                    Some(window),
                    Some(egui_winit),
                    Some(gpu),
                ) = (
                    &self.window,
                    &mut self.egui_winit,
                    &mut self.gpu,
                ) {
                    match &mut self.screen {
                        AppScreen::Menu => {
                            menu::render_menu(window, &self.egui_ctx, egui_winit, gpu, &self.background_texture, self.settings.background_dim);
                            window.request_redraw();
                        }
                        AppScreen::ModeSelect => {
                            menu::render_mode_select(window, &self.egui_ctx, egui_winit, gpu, &self.background_texture, self.settings.background_dim);
                            window.request_redraw();
                        }
                        AppScreen::Settings => {
                            let dim = self.settings.background_dim;
                            let action = settings::render_settings(
                                &self.egui_ctx,
                                &mut self.settings,
                                &mut self.settings_ui,
                                window,
                                egui_winit,
                                gpu,
                                &self.background_texture,
                                dim,
                            );
                            if let SettingsAction::Back = action {
                                self.settings.save();
                                self.screen = AppScreen::Menu;
                                self.needs_settings_apply = true;
                            }
                            window.request_redraw();
                        }
                        AppScreen::Countdown { remaining } => {
                            let elapsed = self.game.last_tick_time.elapsed().as_secs_f32();
                            self.game.last_tick_time = Instant::now();
                            *remaining -= elapsed;
                            if *remaining <= 0.0 {
                                self.game.spawn_random_piece();
                                self.game.play_start_time = Some(Instant::now());
                                self.screen = AppScreen::Playing;
                                window.request_redraw();
                            } else {
                                graphics::render_frame(
                                    &self.game,
                                    window,
                                    &self.egui_ctx,
                                    egui_winit,
                                    gpu,
                                    self.vsync,
                                    self.scale,
                                    self.displayed_fps,
                                    Some(*remaining),
                                    false,
                                    false,
                                    self.game.paused_accumulated,
                                    self.game.pause_start,
                                    &self.background_texture,
                                    self.settings.background_dim,
                                );
                                window.request_redraw();
                            }
                        }
                        AppScreen::Playing => {
                            let elapsed = self.game.last_tick_time.elapsed().as_secs_f32();
                            self.game.last_tick_time = Instant::now();
                            self.game.tick(elapsed);
                            let mode_game_over = match self.game_mode {
                                GameMode::Sprint40L => self.game.total_lines >= 40,
                                GameMode::Blitz => {
                                    if let Some(start) = self.game.play_start_time {
                                        let pause_adjust = self.game.pause_start.map(|ps| ps.elapsed()).unwrap_or(Duration::ZERO);
                                        let total_elapsed = start.elapsed().saturating_sub(self.game.paused_accumulated + pause_adjust);
                                        total_elapsed.as_secs_f32() >= 120.0
                                    } else {
                                        false
                                    }
                                }
                                GameMode::Custom => false,
                            };
                            if self.game.is_game_over() || mode_game_over {
                                if let Some(start) = self.game.play_start_time {
                                    let pause_adjust = self.game.pause_start.map(|ps| ps.elapsed()).unwrap_or(Duration::ZERO);
                                    let total_elapsed = start.elapsed().saturating_sub(self.game.paused_accumulated + pause_adjust);
                                    let ms = total_elapsed.as_millis();
                                    let mins = ms / 60000;
                                    let secs = (ms % 60000) / 1000;
                                    let millis = ms % 1000;
                                    self.end_stats_timer = format!("{:02}:{:02}.{:03}", mins, secs, millis);
                                    let elapsed_secs = total_elapsed.as_secs_f64();
                                    self.end_stats_pps = if elapsed_secs > 0.01 {
                                        self.game.finesse_pieces_placed as f32 / elapsed_secs as f32
                                    } else {
                                        0.0
                                    };
                                    self.end_stats_kpp = if self.game.finesse_pieces_placed > 0 {
                                        self.game.total_keys_pressed as f32 / self.game.finesse_pieces_placed as f32
                                    } else {
                                        0.0
                                    };
                                }
                                self.end_stats_score = self.game.score;
                                self.end_stats_pieces = self.game.finesse_pieces_placed;
                                self.end_stats_lines = self.game.total_lines;
                                self.end_stats_level = self.game.level;
                                self.end_stats_singles = self.game.singles;
                                self.end_stats_doubles = self.game.doubles;
                                self.end_stats_triples = self.game.triples;
                                self.end_stats_quads = self.game.quads;
                                self.end_stats_spins = self.game.total_spins;
                                self.end_stats_max_b2b = self.game.max_b2b;
                                self.end_stats_max_combo = self.game.max_combo;
                                self.end_stats_finesse_faults = self.game.finesse_faults;
                                self.end_stats_total_keys = self.game.total_keys_pressed;
                                if self.game.finesse_pieces_placed > 0 {
                                    self.end_stats_finesse_pct = self.game.finesse_perfect_pieces as f32 / self.game.finesse_pieces_placed as f32 * 100.0;
                                }
                                self.game.play_start_time = None;
                                self.screen = AppScreen::GameOver { remaining: 3.0 };
                                window.request_redraw();
                            } else {
                                graphics::render_frame(
                                    &self.game,
                                    window,
                                    &self.egui_ctx,
                                    egui_winit,
                                    gpu,
                                    self.vsync,
                                    self.scale,
                                    self.displayed_fps,
                                    None,
                                    false,
                                    false,
                                    self.game.paused_accumulated,
                                    self.game.pause_start,
                                    &self.background_texture,
                                    self.settings.background_dim,
                                );
                                window.request_redraw();
                            }
                        }
                        AppScreen::Paused => {
                            graphics::render_frame(
                                &self.game,
                                window,
                                &self.egui_ctx,
                                egui_winit,
                                gpu,
                                self.vsync,
                                self.scale,
                                self.displayed_fps,
                                None,
                                false,
                                true,
                                self.game.paused_accumulated,
                                self.game.pause_start,
                                &self.background_texture,
                                self.settings.background_dim,
                            );
                            window.request_redraw();
                        }
                        AppScreen::GameOver { remaining } => {
                            let elapsed = self.game.last_tick_time.elapsed().as_secs_f32();
                            self.game.last_tick_time = Instant::now();
                            *remaining -= elapsed;
                            if *remaining <= 0.0 {
                                self.screen = AppScreen::EndScreen;
                                window.request_redraw();
                            } else {
                                graphics::render_frame(
                                    &self.game,
                                    window,
                                    &self.egui_ctx,
                                    egui_winit,
                                    gpu,
                                    self.vsync,
                                    self.scale,
                                    self.displayed_fps,
                                    None,
                                    true,
                                    false,
                                    self.game.paused_accumulated,
                                    self.game.pause_start,
                                    &self.background_texture,
                                    self.settings.background_dim,
                                );
                                window.request_redraw();
                            }
                        }
                        AppScreen::EndScreen => {
                            let box_number = match self.game_mode {
                                GameMode::Sprint40L => self.end_stats_timer.clone(),
                                GameMode::Blitz | GameMode::Custom => self.end_stats_score.to_string(),
                            };
                            graphics::render_end_screen(
                                window,
                                &self.egui_ctx,
                                egui_winit,
                                gpu,
                                &box_number,
                                &self.end_stats_timer,
                                self.end_stats_pps,
                                self.end_stats_kpp,
                                self.end_stats_score,
                                self.end_stats_pieces,
                                self.end_stats_lines,
                                self.end_stats_level,
                                self.end_stats_singles,
                                self.end_stats_doubles,
                                self.end_stats_triples,
                                self.end_stats_quads,
                                self.end_stats_spins,
                                self.end_stats_max_b2b,
                                self.end_stats_max_combo,
                                self.end_stats_finesse_faults,
                                self.end_stats_finesse_pct,
                                self.end_stats_total_keys,
                                &self.background_texture,
                                self.settings.background_dim,
                            );
                            window.request_redraw();
                        }
                    }
                }

                if self.needs_settings_apply {
                    self.needs_settings_apply = false;
                    self.apply_settings();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
