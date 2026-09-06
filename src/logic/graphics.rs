use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{Color32, FontId, Pos2, RichText, Stroke, pos2, vec2};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::game::GameState;
use super::piece::Piece;

pub struct GpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub renderer: egui_wgpu::Renderer,
}

impl GpuState {
    pub fn set_present_mode(&mut self, mode: wgpu::PresentMode) {
        self.surface_config.present_mode = mode;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    None,
    Continue,
    Retry,
    ExitToMenu,
}

pub struct PauseButton {
    pub text: &'static str,
    pub top_left: Pos2,
    pub width: f32,
    pub height: f32,
    pub slant: f32,
}

impl PauseButton {
    fn corners(&self) -> [Pos2; 4] {
        [
            Pos2::new(self.top_left.x + self.slant, self.top_left.y),
            Pos2::new(self.top_left.x + self.width, self.top_left.y),
            Pos2::new(self.top_left.x + self.width - self.slant, self.top_left.y + self.height),
            Pos2::new(self.top_left.x, self.top_left.y + self.height),
        ]
    }

    fn contains(&self, pos: Pos2) -> bool {
        let corners = self.corners();
        let mut inside = false;
        let n = corners.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let yi = corners[i].y;
            let yj = corners[j].y;
            if ((yi > pos.y) != (yj > pos.y))
                && (pos.x
                    < (corners[j].x - corners[i].x) * (pos.y - yi) / (yj - yi)
                        + corners[i].x)
            {
                inside = !inside;
            }
        }
        inside
    }
}

pub fn build_pause_buttons(surf_w: f32, surf_h: f32, sf: f32) -> Vec<PauseButton> {
    let logical_w = surf_w / sf;
    let logical_h = surf_h / sf;

    let btn_w = logical_w / 4.0;
    let btn_h = 50.0;
    let slant = 25.0;
    let gap = 20.0;
    let start_y = logical_h / 2.0 - btn_h - gap;
    let start_x = 40.0;

    vec![
        PauseButton { text: "Continue",    top_left: Pos2::new(start_x, start_y),                    width: btn_w, height: btn_h, slant },
        PauseButton { text: "Retry",       top_left: Pos2::new(start_x, start_y + btn_h + gap),      width: btn_w, height: btn_h, slant },
        PauseButton { text: "Exit to Menu", top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 2.0), width: btn_w, height: btn_h, slant },
    ]
}

pub fn handle_pause_click(pos: Pos2, buttons: &[PauseButton]) -> PauseAction {
    for (i, btn) in buttons.iter().enumerate() {
        if btn.contains(pos) {
            return match i {
                0 => PauseAction::Continue,
                1 => PauseAction::Retry,
                2 => PauseAction::ExitToMenu,
                _ => PauseAction::None,
            };
        }
    }
    PauseAction::None
}

pub fn init_graphics(
    event_loop: &ActiveEventLoop,
    egui_ctx: &egui::Context,
    vsync: bool,
    width: u32,
    height: u32,
) -> (Arc<Window>, GpuState, egui_winit::State) {
    let window_attrs = winit::window::WindowAttributes::default()
        .with_title("Tetrino")
        .with_inner_size(winit::dpi::LogicalSize::new(width, height));

    let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("Failed to find a suitable GPU adapter");
    
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("tetrino device"),
            required_features: wgpu::Features::default(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .expect("Failed to create device");

    let size = window.inner_size();
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(caps.formats[0]);

    let present_mode = if vsync {
        wgpu::PresentMode::Fifo
    } else {
        wgpu::PresentMode::Immediate
    };

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 1,
    };
    surface.configure(&device, &surface_config);

    let renderer = egui_wgpu::Renderer::new(&device, format, None, 1, true);

    let egui_winit = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        event_loop,
        None,
        event_loop.system_theme(),
        Some(device.limits().max_texture_dimension_2d as usize),
    );

    (
        window,
        GpuState {
            surface,
            device,
            queue,
            surface_config,
            renderer,
        },
        egui_winit,
    )
}

pub fn render_background(
    ctx: &egui::Context,
    texture: &Option<egui::TextureHandle>,
    surf_w: f32,
    surf_h: f32,
    dim: f32,
) {
    if let Some(handle) = texture {
        let layer = egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("background"),
        );
        let painter = ctx.layer_painter(layer);
        let uv = egui::Rect::from_min_max(
            pos2(0.0, 0.0),
            pos2(1.0, 1.0),
        );
        let tex_aspect = handle.aspect_ratio();
        let screen_aspect = surf_w / surf_h;
        let (draw_w, draw_h) = if tex_aspect > screen_aspect {
            let h = surf_h;
            let w = h * tex_aspect;
            (w, h)
        } else {
            let w = surf_w;
            let h = w / tex_aspect;
            (w, h)
        };
        let x = (surf_w - draw_w) / 2.0;
        let y = (surf_h - draw_h) / 2.0;
        let rect = egui::Rect::from_min_size(
            pos2(x, y),
            vec2(draw_w, draw_h),
        );
        painter.image(handle.id(), rect, uv, Color32::WHITE);

        if dim > 0.0 {
            let screen_rect = egui::Rect::from_min_size(
                pos2(0.0, 0.0),
                vec2(surf_w, surf_h),
            );
            painter.rect_filled(screen_rect, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, (dim * 255.0) as u8));
        }
    }
}

pub fn render_frame(
    game: &GameState,
    window: &Window,
    egui_ctx: &egui::Context,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    vsync: bool,
    scale: f32,
    fps: usize,
    countdown: Option<f32>,
    game_over: bool,
    paused: bool,
    paused_accumulated: Duration,
    pause_start: Option<Instant>,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) {
    let raw_input = egui_winit.take_egui_input(window);

    let backend = format!("{:?}", gpu.surface_config.format);
    let surf_w = gpu.surface_config.width;
    let surf_h = gpu.surface_config.height;
    let sf = window.scale_factor() as f32;

    let grid = game.grid;
    let upcoming: Vec<i32> = game.bag_queue.iter().take(5).copied().collect();
    let level = game.level;
    let active_piece = game.active_piece;
    let active_cells = game.active_cells;
    let active_pos = game.active_pos;
    let last_clear = game.last_clear;
    let score = game.score;
    let combo = game.combo;
    let b2b = game.b2b;
    let hold_piece = game.hold_piece;
    let last_spin_piece = game.last_spin_piece;
    let last_spin_mini = game.last_spin_mini;
    let finesse_faults = game.finesse_faults;
    let finesse_pieces_placed = game.finesse_pieces_placed;
    let finesse_perfect_pieces = game.finesse_perfect_pieces;
    let play_start_time = game.play_start_time;

    let ghost_pos = game.compute_ghost_pos();

    let full_output = egui_ctx.run(raw_input, |ctx| {
        render_background(ctx, background_texture, surf_w as f32, surf_h as f32, dim);

        let vsync_str = if vsync { "ON" } else { "OFF" };
        let vsync_color = if vsync {
            Color32::from_rgb(80, 220, 80)
        } else {
            Color32::from_rgb(220, 80, 80)
        };

        egui::Area::new(egui::Id::new("fps_overlay"))
            .fixed_pos((8.0, 8.0))
            .interactable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("FPS: {fps}"))
                            .font(FontId::proportional(20.0))
                            .color(Color32::WHITE),
                    );
                    ui.label(
                        RichText::new(format!("VSync: {vsync_str}"))
                            .font(FontId::proportional(16.0))
                            .color(vsync_color),
                    );
                    ui.label(
                        RichText::new(format!("Backend: {backend}"))
                            .font(FontId::proportional(14.0))
                            .color(Color32::from_rgb(160, 160, 160)),
                    );
                    ui.label(
                        RichText::new(format!("Level: {level}"))
                            .font(FontId::proportional(16.0))
                            .color(Color32::from_rgb(200, 200, 255)),
                    );
                });
            });

        let cell_size = 24.0 * scale;
        let grid_w = 10.0 * cell_size;
        let grid_h = 20.0 * cell_size;
        let logical_w = surf_w as f32 / sf;
        let logical_h = surf_h as f32 / sf;
        let off_x = (logical_w - grid_w) / 2.0;
        let off_y = (logical_h - grid_h) / 2.0;

        let layer = egui::LayerId::new(egui::Order::Background, egui::Id::new("grid"));
        let painter = ctx.layer_painter(layer);
        let stroke = Stroke::new(1.0_f32, Color32::from_rgb(100, 100, 100));

        for hidden_row in 0..3 {
            for col in 0..10 {
                if let Some(color) = grid[hidden_row][col] {
                    let rect = egui::Rect::from_min_size(
                        pos2(
                            off_x + col as f32 * cell_size,
                            off_y + (hidden_row as f32 - 3.0) * cell_size,
                        ),
                        vec2(cell_size, cell_size),
                    );
                    painter.rect_filled(rect, 0.0, color);
                }
            }
        }

        for row in 0..20 {
            for col in 0..10 {
                let rect = egui::Rect::from_min_size(
                    pos2(
                        off_x + col as f32 * cell_size,
                        off_y + row as f32 * cell_size,
                    ),
                    vec2(cell_size, cell_size),
                );
                if let Some(color) = grid[row + 3][col] {
                    painter.rect_filled(rect, 0.0, color);
                }
                painter.rect_stroke(rect, 0.0, stroke);
            }
        }

        if let (Some(_piece), Some(gp)) = (active_piece, ghost_pos) {
            let ghost_color = Color32::from_rgba_premultiplied(180, 180, 180, 128);
            for &(dr, dc) in &active_cells {
                let r = gp.0 + dr - 3;
                let c = gp.1 + dc;
                if r >= 0 && r < 20 && c >= 0 && c < 10 {
                    let rect = egui::Rect::from_min_size(
                        pos2(off_x + c as f32 * cell_size, off_y + r as f32 * cell_size),
                        vec2(cell_size, cell_size),
                    );
                    painter.rect_filled(rect, 0.0, ghost_color);
                    painter.rect_stroke(rect, 0.0, stroke);
                }
            }
        }

        if let Some(piece) = active_piece {
            let color = piece.color();
            for &(dr, dc) in &active_cells {
                let r = active_pos.0 + dr - 3;
                let c = active_pos.1 + dc;
                if r >= 0 && r < 20 && c >= 0 && c < 10 {
                    let rect = egui::Rect::from_min_size(
                        pos2(off_x + c as f32 * cell_size, off_y + r as f32 * cell_size),
                        vec2(cell_size, cell_size),
                    );
                    painter.rect_filled(rect, 0.0, color);
                    painter.rect_stroke(rect, 0.0, stroke);
                }
            }
        }

        let preview_rect = egui::Rect::from_min_size(
            pos2(off_x + grid_w + 2.0 * cell_size, off_y),
            vec2(grid_w / 2.0, grid_h * 0.75),
        );
        painter.rect_stroke(preview_rect, 0.0, stroke);

        let preview_cell = cell_size;
        let preview_origin_x = off_x + grid_w + 2.0 * cell_size;
        let preview_origin_y = off_y;
        for i in 0..upcoming.len().min(5) {
            if let Some(piece) = Piece::from_id(upcoming[i]) {
                let cells = piece.spawn_cells();
                let color = piece.color();
                let max_col = cells.iter().map(|c| c.1).max().unwrap_or(0) as f32;
                let piece_w = (max_col + 1.0) * preview_cell;
                let box_w = grid_w / 2.0;
                let x_offset = (box_w - piece_w) / 2.0;
                let y_base = preview_origin_y + i as f32 * 3.0 * preview_cell;
                for &(row, col) in &cells {
                    let rect = egui::Rect::from_min_size(
                        pos2(
                            preview_origin_x + x_offset + col as f32 * preview_cell,
                            y_base + row as f32 * preview_cell,
                        ),
                        vec2(preview_cell, preview_cell),
                    );
                    painter.rect_filled(rect, 0.0, color);
                    painter.rect_stroke(rect, 0.0, stroke);
                }
            }
        }

        let score_y = off_y + grid_h * 0.75 + 12.0;
        let score_galley =
            painter.layout(format!("Score: {}", score), FontId::proportional(20.0), Color32::WHITE, grid_w);
        painter.galley(pos2(off_x + grid_w + 2.0 * cell_size, score_y), score_galley, Color32::WHITE);

        let combo_galley = painter.layout(
            format!("Combo: {}", combo),
            FontId::proportional(16.0),
            if combo > 1 {
                Color32::from_rgb(100, 255, 100)
            } else {
                Color32::from_rgb(160, 160, 160)
            },
            grid_w,
        );
        painter.galley(
            pos2(off_x + grid_w + 2.0 * cell_size, score_y + 28.0),
            combo_galley,
            Color32::WHITE,
        );

        let b2b_galley = painter.layout(
            format!("B2B: {}", if b2b > 1 { b2b - 1 } else { 0 }),
            FontId::proportional(16.0),
            if b2b > 1 {
                Color32::from_rgb(255, 200, 50)
            } else {
                Color32::from_rgb(160, 160, 160)
            },
            grid_w,
        );
        painter.galley(
            pos2(off_x + grid_w + 2.0 * cell_size, score_y + 50.0),
            b2b_galley,
            Color32::WHITE,
        );

        let left_size = grid_h / 4.0;
        let left_rect = egui::Rect::from_min_size(
            pos2(off_x - 2.0 * cell_size - left_size, off_y),
            vec2(left_size, left_size),
        );
        painter.rect_stroke(left_rect, 0.0, stroke);

        if let Some(piece) = hold_piece {
            let cells = piece.spawn_cells();
            let color = piece.color();
            let max_col = cells.iter().map(|c| c.1).max().unwrap_or(0) as f32;
            let piece_w = (max_col + 1.0) * cell_size;
            let x_offset = (left_size - piece_w) / 2.0;
            let y_offset = (left_size - 2.0 * cell_size) / 2.0;
            for &(row, col) in &cells {
                let rect = egui::Rect::from_min_size(
                    pos2(
                        left_rect.min.x + x_offset + col as f32 * cell_size,
                        left_rect.min.y + y_offset + row as f32 * cell_size,
                    ),
                    vec2(cell_size, cell_size),
                );
                painter.rect_filled(rect, 0.0, color);
                painter.rect_stroke(rect, 0.0, stroke);
            }
        }

        let finesse_x = off_x - 2.0 * cell_size - left_size;
        let finesse_pct = if finesse_pieces_placed > 0 {
            format!("{:.1}%", finesse_perfect_pieces as f32 / finesse_pieces_placed as f32 * 100.0)
        } else {
            "--%".to_string()
        };
        let pct_galley = painter.layout(
            format!("Finesse: {finesse_pct}"),
            FontId::proportional(16.0),
            Color32::WHITE,
            left_size,
        );
        let pct_y = off_y + grid_h - 50.0;
        painter.galley(pos2(finesse_x, pct_y), pct_galley, Color32::WHITE);

        if let Some(start) = play_start_time {
            let pause_adjust = pause_start.map(|ps| ps.elapsed()).unwrap_or(Duration::ZERO);
            let total_elapsed = start.elapsed().saturating_sub(paused_accumulated + pause_adjust);
            let elapsed_secs = total_elapsed.as_secs_f64();
            let ms = total_elapsed.as_millis();
            let mins = ms / 60000;
            let secs = (ms % 60000) / 1000;
            let millis = ms % 1000;
            let timer_text = format!("{:02}:{:02}.{:03}", mins, secs, millis);
            let timer_galley = painter.layout(
                timer_text,
                FontId::proportional(16.0),
                Color32::WHITE,
                left_size,
            );
            let timer_y = off_y + grid_h - 72.0;
            painter.galley(pos2(finesse_x, timer_y), timer_galley, Color32::WHITE);

            let pieces_y = off_y + grid_h - 94.0;
            let pieces_galley = painter.layout(
                format!("{}", finesse_pieces_placed),
                FontId::proportional(24.0),
                Color32::WHITE,
                left_size,
            );
            painter.galley(pos2(finesse_x, pieces_y), pieces_galley.clone(), Color32::WHITE);

            let pps = if elapsed_secs > 0.01 {
                finesse_pieces_placed as f64 / elapsed_secs
            } else {
                0.0
            };
            let pps_x = finesse_x + pieces_galley.size().x + 6.0;
            let pps_galley = painter.layout(
                format!("{:.1}/s", pps),
                FontId::proportional(14.0),
                Color32::from_rgb(180, 180, 180),
                left_size,
            );
            let pps_y = pieces_y + 10.0;
            painter.galley(pos2(pps_x, pps_y), pps_galley, Color32::WHITE);
        }

        let faults_color = if finesse_faults > 0 {
            Color32::from_rgb(255, 100, 100)
        } else {
            Color32::from_rgb(100, 255, 100)
        };
        let faults_galley = painter.layout(
            format!("Faults: {finesse_faults}"),
            FontId::proportional(14.0),
            faults_color,
            left_size,
        );
        let faults_y = off_y + grid_h - 28.0;
        painter.galley(pos2(finesse_x, faults_y), faults_galley, Color32::WHITE);

        if last_clear > 0 {
            let clear_text = match last_clear {
                1 => "Single",
                2 => "Double",
                3 => "Triple",
                4 => "Quad",
                _ => "",
            };
            let clear_color = match last_clear {
                1 => Color32::from_rgb(255, 255, 255),
                2 => Color32::from_rgb(100, 255, 100),
                3 => Color32::from_rgb(100, 200, 255),
                4 => Color32::from_rgb(255, 200, 50),
                _ => Color32::WHITE,
            };
            let spin_color = Color32::from_rgb(255, 180, 0);
            let base_x = off_x - 2.0 * cell_size - left_size;
            let mut y = off_y + left_size + 8.0;

            if let Some(piece) = last_spin_piece {
                let prefix = if last_spin_mini { "Mini " } else { "" };
                let piece_letter = match piece {
                    Piece::T => "T",
                    Piece::J => "J",
                    Piece::L => "L",
                    Piece::Z => "Z",
                    Piece::S => "S",
                    Piece::I => "I",
                    Piece::O => "O",
                };
                let spin_text = format!("{prefix}{piece_letter}-Spin");
                let spin_galley = painter.layout(spin_text, FontId::proportional(20.0), spin_color, left_size);
                let spin_x = base_x + (left_size - spin_galley.size().x) / 2.0;
                painter.galley(pos2(spin_x, y), spin_galley, Color32::WHITE);
                y += 24.0;
            }

            if last_clear > 0 {
                let galley = painter.layout(clear_text.to_string(), FontId::proportional(24.0), clear_color, left_size);
                let text_x = base_x + (left_size - galley.size().x) / 2.0;
                painter.galley(pos2(text_x, y), galley, Color32::WHITE);
            }
        }

        if let Some(remaining) = countdown {
            let number = if remaining > 2.0 {
                "3"
            } else if remaining > 1.0 {
                "2"
            } else {
                "1"
            };
            let count_galley = painter.layout(
                number.to_string(),
                FontId::proportional(96.0),
                Color32::from_rgb(255, 255, 0),
                logical_w,
            );
            let cx = (logical_w - count_galley.size().x) / 2.0;
            let cy = off_y + (grid_h - count_galley.size().y) / 2.0;
            painter.galley(Pos2::new(cx, cy), count_galley, Color32::WHITE);
        }

        if game_over {
            let go_galley = painter.layout(
                "Game Over".to_string(),
                FontId::proportional(64.0),
                Color32::from_rgb(255, 50, 50),
                logical_w,
            );
            let cx = (logical_w - go_galley.size().x) / 2.0;
            let cy = off_y + (grid_h - go_galley.size().y) / 2.0;
            painter.galley(Pos2::new(cx, cy), go_galley, Color32::WHITE);
        }

        if paused {
            let dim = egui::Rect::from_min_max(
                pos2(0.0, 0.0),
                pos2(logical_w, logical_h),
            );
            painter.rect_filled(dim, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 51));

            let pause_galley = painter.layout(
                "Paused".to_string(),
                FontId::proportional(32.0),
                Color32::WHITE,
                logical_w,
            );
            painter.galley(pos2(40.0, 40.0), pause_galley, Color32::WHITE);

            let buttons = build_pause_buttons(surf_w as f32, surf_h as f32, sf);
            let cyan = Color32::from_rgb(0, 150, 160);
            let dark_cyan = Color32::from_rgb(0, 120, 130);
            let stroke = Stroke::new(2.0_f32, dark_cyan);

            for button in &buttons {
                let corners = button.corners();
                let points: Vec<Pos2> = corners.to_vec();
                let path = egui::epaint::PathShape::convex_polygon(points, cyan, stroke);
                painter.add(path);

                let btn_center_x = button.top_left.x + button.width / 2.0;
                let btn_center_y = button.top_left.y + button.height / 2.0;
                let text_galley = painter.layout(
                    button.text.to_string(),
                    FontId::proportional(24.0),
                    Color32::WHITE,
                    button.width,
                );
                let text_x = btn_center_x - text_galley.size().x / 2.0;
                let text_y = btn_center_y - text_galley.size().y / 2.0;
                painter.galley(Pos2::new(text_x, text_y), text_galley, Color32::WHITE);
            }
        }
    });

    egui_winit.handle_platform_output(window, full_output.platform_output);

    let clipped_primitives = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [gpu.surface_config.width, gpu.surface_config.height],
        pixels_per_point: window.scale_factor() as f32,
    };

    let surface_output = match gpu.surface.get_current_texture() {
        Ok(o) => o,
        Err(wgpu::SurfaceError::Outdated) => {
            let size = window.inner_size();
            gpu.resize(size.width, size.height);
            window.request_redraw();
            return;
        }
        Err(e) => {
            eprintln!("Surface error: {e}");
            window.request_redraw();
            return;
        }
    };

    let view = surface_output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("egui encoder"),
        });

    for (id, delta) in &full_output.textures_delta.set {
        gpu.renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
    }

    let user_cmd_bufs = gpu.renderer.update_buffers(
        &gpu.device,
        &gpu.queue,
        &mut encoder,
        &clipped_primitives,
        &screen_descriptor,
    );

    {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        let mut rp = render_pass.forget_lifetime();
        gpu.renderer
            .render(&mut rp, &clipped_primitives, &screen_descriptor);
    }

    gpu.queue
        .submit(user_cmd_bufs.into_iter().chain([encoder.finish()]));
    surface_output.present();

    for id in &full_output.textures_delta.free {
        gpu.renderer.free_texture(id);
    }
}

pub fn build_end_screen_exit_button(surf_w: f32, _surf_h: f32, sf: f32) -> PauseButton {
    let logical_w = surf_w / sf;
    let btn_w = logical_w / 4.0;
    let btn_h = 50.0;
    let slant = 25.0;
    PauseButton {
        text: "Exit",
        top_left: Pos2::new(40.0, 40.0),
        width: btn_w,
        height: btn_h,
        slant,
    }
}

pub fn handle_end_screen_click(pos: Pos2, button: &PauseButton) -> bool {
    button.contains(pos)
}

pub fn render_end_screen(
    window: &Window,
    egui_ctx: &egui::Context,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    box_number: &str,
    timer: &str,
    pps: f32,
    kpp: f32,
    score: u64,
    pieces: u32,
    lines: u32,
    level: i32,
    singles: u32,
    doubles: u32,
    triples: u32,
    quads: u32,
    spins: u32,
    max_b2b: i32,
    max_combo: i32,
    finesse_faults: u32,
    finesse_pct: f32,
    total_keys: u32,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) {
    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width;
    let surf_h = gpu.surface_config.height;
    let sf = window.scale_factor() as f32;

    let exit_button = build_end_screen_exit_button(surf_w as f32, surf_h as f32, sf);

    let full_output = egui_ctx.run(raw_input, |ctx| {
        render_background(ctx, background_texture, surf_w as f32, surf_h as f32, dim);

        let logical_w = surf_w as f32 / sf;
        let logical_h = surf_h as f32 / sf;

        let layer = egui::LayerId::new(egui::Order::Background, egui::Id::new("endscreen"));
        let painter = ctx.layer_painter(layer);

        let box_w = logical_w / 3.0;
        let box_h = logical_h / 4.0;
        let box_x = (logical_w - box_w) / 2.0;
        let box_y = logical_h / 6.0;
        let box_rect = egui::Rect::from_min_size(
            pos2(box_x, box_y),
            vec2(box_w, box_h),
        );
        painter.rect_filled(box_rect, 0.0, Color32::from_rgb(20, 20, 20));
        painter.rect_stroke(box_rect, 0.0, Stroke::new(1.0_f32, Color32::from_rgb(100, 100, 100)));

        let score_galley = painter.layout(
            box_number.to_string(),
            FontId::proportional(64.0),
            Color32::WHITE,
            box_w,
        );
        let score_x = box_x + (box_w - score_galley.size().x) / 2.0;
        let score_y = box_y + (box_h - score_galley.size().y) / 2.0;
        painter.galley(Pos2::new(score_x, score_y), score_galley, Color32::WHITE);

        let cyan = Color32::from_rgb(0, 150, 160);
        let dark_cyan = Color32::from_rgb(0, 120, 130);
        let stroke = Stroke::new(2.0_f32, dark_cyan);

        let corners = exit_button.corners();
        let points: Vec<Pos2> = corners.to_vec();
        let path = egui::epaint::PathShape::convex_polygon(points, cyan, stroke);
        painter.add(path);

        let btn_center_x = exit_button.top_left.x + exit_button.width / 2.0;
        let btn_center_y = exit_button.top_left.y + exit_button.height / 2.0;
        let text_galley = painter.layout(
            exit_button.text.to_string(),
            FontId::proportional(24.0),
            Color32::WHITE,
            exit_button.width,
        );
        let text_x = btn_center_x - text_galley.size().x / 2.0;
        let text_y = btn_center_y - text_galley.size().y / 2.0;
        painter.galley(Pos2::new(text_x, text_y), text_galley, Color32::WHITE);

        let stats = [
            ("Pieces Placed", format!("{}", pieces)),
            ("PPS", format!("{:.2}", pps)),
            ("Keys/Piece", format!("{:.3}", kpp)),
            ("Total Keys", format!("{}", total_keys)),
            ("Time", timer.to_string()),
            ("Max B2B", format!("{}", max_b2b)),
            ("Max Combo", format!("{}", max_combo)),
            ("Finesse", format!("{:.1}%", finesse_pct)),
            ("Finesse Faults", format!("{}", finesse_faults)),
            ("Score", format!("{}", score)),
            ("Spins", format!("{}", spins)),
            ("Singles", format!("{}", singles)),
            ("Doubles", format!("{}", doubles)),
            ("Triples", format!("{}", triples)),
            ("Quads", format!("{}", quads)),
            ("Max Level", format!("{}", level)),
            ("Lines Cleared", format!("{}", lines)),
        ];

        let stats_start_y = box_y + box_h + 20.0;
        let row_height = 24.0;
        let font = FontId::proportional(16.0);

        for (i, (label, value)) in stats.iter().enumerate() {
            let y = stats_start_y + i as f32 * row_height;
            let label_galley = painter.layout(
                label.to_string(),
                font.clone(),
                Color32::WHITE,
                box_w,
            );
            painter.galley(Pos2::new(box_x, y), label_galley, Color32::WHITE);

            let value_galley = painter.layout(
                value.to_string(),
                font.clone(),
                Color32::WHITE,
                box_w,
            );
            let value_x = box_x + box_w - value_galley.size().x;
            painter.galley(Pos2::new(value_x, y), value_galley, Color32::WHITE);
        }
    });

    egui_winit.handle_platform_output(window, full_output.platform_output);

    let clipped_primitives = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [gpu.surface_config.width, gpu.surface_config.height],
        pixels_per_point: window.scale_factor() as f32,
    };

    let surface_output = match gpu.surface.get_current_texture() {
        Ok(o) => o,
        Err(wgpu::SurfaceError::Outdated) => {
            let size = window.inner_size();
            gpu.resize(size.width, size.height);
            window.request_redraw();
            return;
        }
        Err(e) => {
            eprintln!("Surface error: {e}");
            window.request_redraw();
            return;
        }
    };

    let view = surface_output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("endscreen encoder"),
        });

    for (id, delta) in &full_output.textures_delta.set {
        gpu.renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
    }

    let user_cmd_bufs = gpu.renderer.update_buffers(
        &gpu.device,
        &gpu.queue,
        &mut encoder,
        &clipped_primitives,
        &screen_descriptor,
    );

    {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("endscreen render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        let mut rp = render_pass.forget_lifetime();
        gpu.renderer
            .render(&mut rp, &clipped_primitives, &screen_descriptor);
    }

    gpu.queue
        .submit(user_cmd_bufs.into_iter().chain([encoder.finish()]));
    surface_output.present();

    for id in &full_output.textures_delta.free {
        gpu.renderer.free_texture(id);
    }
}
