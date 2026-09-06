use std::fs;

use egui::{self, Color32, FontId, Pos2, RichText, ScrollArea};
use serde::{Deserialize, Serialize};

use winit::keyboard::Key;
use winit::window::Window;

use super::graphics::{self, GpuState};

const CONFIG_PATH: &str = "settings.toml";

pub const FPS_CAPS: &[u32] = &[30, 60, 120, 144, 240];
pub const ASPECT_RATIOS: &[&str] = &["16:9", "16:10", "4:3", "21:9"];

pub fn resolutions_for_ratio(ratio: &str) -> Vec<(u32, u32)> {
    match ratio {
        "16:9" => vec![(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)],
        "16:10" => vec![(1280, 800), (1920, 1200), (2560, 1600), (3840, 2400)],
        "4:3" => vec![(1024, 768), (1280, 960), (1600, 1200), (2048, 1536)],
        "21:9" => vec![(2560, 1080), (3440, 1440), (5120, 2160)],
        _ => vec![(1280, 720)],
    }
}

pub fn resolution_label(res: (u32, u32)) -> String {
    format!("{}×{}", res.0, res.1)
}

pub fn key_display_name(key: &str) -> String {
    match key {
        " " => "Space".to_string(),
        s if s.len() == 1 => s.to_uppercase(),
        s => s.to_string(),
    }
}

pub fn key_to_string(key: &Key) -> String {
    match key {
        Key::Named(winit::keyboard::NamedKey::Space) => " ".to_string(),
        Key::Character(s) => s.to_string(),
        Key::Named(winit::keyboard::NamedKey::ArrowLeft) => "ArrowLeft".to_string(),
        Key::Named(winit::keyboard::NamedKey::ArrowRight) => "ArrowRight".to_string(),
        Key::Named(winit::keyboard::NamedKey::ArrowUp) => "ArrowUp".to_string(),
        Key::Named(winit::keyboard::NamedKey::ArrowDown) => "ArrowDown".to_string(),
        Key::Named(winit::keyboard::NamedKey::Shift) => "Shift".to_string(),
        Key::Named(winit::keyboard::NamedKey::Control) => "Control".to_string(),
        Key::Named(winit::keyboard::NamedKey::Alt) => "Alt".to_string(),
        Key::Named(winit::keyboard::NamedKey::Tab) => "Tab".to_string(),
        Key::Named(winit::keyboard::NamedKey::Escape) => "Escape".to_string(),
        Key::Named(winit::keyboard::NamedKey::Enter) => "Enter".to_string(),
        Key::Named(winit::keyboard::NamedKey::Backspace) => "Backspace".to_string(),
        Key::Named(winit::keyboard::NamedKey::Delete) => "Delete".to_string(),
        Key::Named(winit::keyboard::NamedKey::F1) => "F1".to_string(),
        Key::Named(winit::keyboard::NamedKey::F2) => "F2".to_string(),
        Key::Named(winit::keyboard::NamedKey::F3) => "F3".to_string(),
        Key::Named(winit::keyboard::NamedKey::F4) => "F4".to_string(),
        Key::Named(winit::keyboard::NamedKey::F5) => "F5".to_string(),
        Key::Named(winit::keyboard::NamedKey::F6) => "F6".to_string(),
        Key::Named(winit::keyboard::NamedKey::F7) => "F7".to_string(),
        Key::Named(winit::keyboard::NamedKey::F8) => "F8".to_string(),
        Key::Named(winit::keyboard::NamedKey::F9) => "F9".to_string(),
        Key::Named(winit::keyboard::NamedKey::F10) => "F10".to_string(),
        Key::Named(winit::keyboard::NamedKey::F11) => "F11".to_string(),
        Key::Named(winit::keyboard::NamedKey::F12) => "F12".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub arr: f32,
    pub das: f32,
    pub dcd: f32,
    pub sdf: f32,
    pub infinite_sdf: bool,

    pub key_move_left: String,
    pub key_move_right: String,
    pub key_soft_drop: String,
    pub key_hard_drop: String,
    pub key_rotate_cw: String,
    pub key_rotate_ccw: String,
    pub key_rotate_180: String,
    pub key_hold: String,

    pub vsync: bool,
    pub fps_cap: Option<u32>,
    pub resolution: (u32, u32),
    pub aspect_ratio: String,
    pub fullscreen: bool,
    pub background_path: Option<String>,
    pub background_dim: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            arr: 1.0,
            das: 100.0,
            dcd: 50.0,
            sdf: 1.0,
            infinite_sdf: false,

            key_move_left: "a".to_string(),
            key_move_right: "d".to_string(),
            key_soft_drop: "s".to_string(),
            key_hard_drop: " ".to_string(),
            key_rotate_cw: "l".to_string(),
            key_rotate_ccw: "j".to_string(),
            key_rotate_180: "k".to_string(),
            key_hold: "f".to_string(),

            vsync: true,
            fps_cap: None,
            resolution: (1280, 720),
            aspect_ratio: "16:9".to_string(),
            fullscreen: false,
            background_path: None,
            background_dim: 0.0,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        match fs::read_to_string(CONFIG_PATH) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        if let Ok(contents) = toml::to_string_pretty(self) {
            let _ = fs::write(CONFIG_PATH, contents);
        }
    }

    pub fn key_matches(&self, key: &Key, action: &str) -> bool {
        let binding = match action {
            "move_left" => &self.key_move_left,
            "move_right" => &self.key_move_right,
            "soft_drop" => &self.key_soft_drop,
            "hard_drop" => &self.key_hard_drop,
            "rotate_cw" => &self.key_rotate_cw,
            "rotate_ccw" => &self.key_rotate_ccw,
            "rotate_180" => &self.key_rotate_180,
            "hold" => &self.key_hold,
            _ => return false,
        };
        match key {
            Key::Character(s) => s.as_ref().eq_ignore_ascii_case(binding.as_str()),
            Key::Named(winit::keyboard::NamedKey::Space) => binding == " ",
            Key::Named(winit::keyboard::NamedKey::ArrowLeft) => binding == "ArrowLeft",
            Key::Named(winit::keyboard::NamedKey::ArrowRight) => binding == "ArrowRight",
            Key::Named(winit::keyboard::NamedKey::ArrowUp) => binding == "ArrowUp",
            Key::Named(winit::keyboard::NamedKey::ArrowDown) => binding == "ArrowDown",
            Key::Named(winit::keyboard::NamedKey::Shift) => binding == "Shift",
            Key::Named(winit::keyboard::NamedKey::Control) => binding == "Control",
            Key::Named(winit::keyboard::NamedKey::Alt) => binding == "Alt",
            Key::Named(winit::keyboard::NamedKey::Tab) => binding == "Tab",
            Key::Named(winit::keyboard::NamedKey::Escape) => binding == "Escape",
            Key::Named(winit::keyboard::NamedKey::Enter) => binding == "Enter",
            Key::Named(winit::keyboard::NamedKey::Backspace) => binding == "Backspace",
            Key::Named(winit::keyboard::NamedKey::Delete) => binding == "Delete",
            Key::Named(winit::keyboard::NamedKey::F1) => binding == "F1",
            Key::Named(winit::keyboard::NamedKey::F2) => binding == "F2",
            Key::Named(winit::keyboard::NamedKey::F3) => binding == "F3",
            Key::Named(winit::keyboard::NamedKey::F4) => binding == "F4",
            Key::Named(winit::keyboard::NamedKey::F5) => binding == "F5",
            Key::Named(winit::keyboard::NamedKey::F6) => binding == "F6",
            Key::Named(winit::keyboard::NamedKey::F7) => binding == "F7",
            Key::Named(winit::keyboard::NamedKey::F8) => binding == "F8",
            Key::Named(winit::keyboard::NamedKey::F9) => binding == "F9",
            Key::Named(winit::keyboard::NamedKey::F10) => binding == "F10",
            Key::Named(winit::keyboard::NamedKey::F11) => binding == "F11",
            Key::Named(winit::keyboard::NamedKey::F12) => binding == "F12",
            _ => false,
        }
    }
}

pub enum SettingsAction {
    None,
    Back,
}

pub struct SettingsUiState {
    pub listening_for: Option<String>,
    pub pick_background: bool,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            listening_for: None,
            pick_background: false,
        }
    }
}

pub fn render_settings(
    ctx: &egui::Context,
    settings: &mut Settings,
    ui_state: &mut SettingsUiState,
    window: &Window,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) -> SettingsAction {
    let mut action = SettingsAction::None;

    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width as f32;
    let surf_h = gpu.surface_config.height as f32;

    let full_output = ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w, surf_h, dim);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            let header_galley = ui.painter().layout(
                "Settings".to_string(),
                FontId::proportional(40.0),
                Color32::WHITE,
                ui.available_width(),
            );
            let header_x = (ui.available_width() - header_galley.size().x) / 2.0;
            ui.painter().galley(
                Pos2::new(ui.clip_rect().min.x + header_x, ui.clip_rect().min.y + 10.0),
                header_galley,
                Color32::WHITE,
            );
            ui.add_space(50.0);

            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 500.0).max(0.0) / 2.0);
                    ui.vertical(|ui| {
                        ui.set_min_width(500.0);

                        let header_color = Color32::from_rgb(0, 200, 220);

                        ui.label(RichText::new("Gameplay").font(FontId::proportional(24.0)).color(header_color));
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("ARR (ms):").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            ui.add(egui::Slider::new(&mut settings.arr, 0.0..=10.0).step_by(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("DAS (ms):").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            ui.add(egui::Slider::new(&mut settings.das, 0.0..=300.0).step_by(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("DCD (ms):").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            ui.add(egui::Slider::new(&mut settings.dcd, 0.0..=100.0).step_by(1.0));
                        });
                    ui.checkbox(&mut settings.infinite_sdf, RichText::new("Infinite SDF (instant drop)").font(FontId::proportional(16.0)).color(Color32::WHITE));
                    if !settings.infinite_sdf {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("SDF:").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            ui.add(egui::Slider::new(&mut settings.sdf, 1.0..=100.0).step_by(1.0));
                        });
                    }

                        ui.add_space(16.0);
                        ui.label(RichText::new("Keybinds").font(FontId::proportional(24.0)).color(header_color));
                        ui.add_space(4.0);

                        let keybinds = [
                            ("move_left", "Move Left", &settings.key_move_left.clone()),
                            ("move_right", "Move Right", &settings.key_move_right.clone()),
                            ("soft_drop", "Soft Drop", &settings.key_soft_drop.clone()),
                            ("hard_drop", "Hard Drop", &settings.key_hard_drop.clone()),
                            ("rotate_cw", "Rotate CW", &settings.key_rotate_cw.clone()),
                            ("rotate_ccw", "Rotate CCW", &settings.key_rotate_ccw.clone()),
                            ("rotate_180", "Rotate 180", &settings.key_rotate_180.clone()),
                            ("hold", "Hold", &settings.key_hold.clone()),
                        ];

                        for (act, label, current) in &keybinds {
                            let is_listening = ui_state.listening_for.as_deref() == Some(*act);
                            let display = if is_listening {
                                "...".to_string()
                            } else {
                                key_display_name(current)
                            };

                            let btn_text = format!("{label}: [{display}]");
                            let btn = if is_listening {
                                ui.button(RichText::new(&btn_text).color(Color32::from_rgb(255, 200, 0)).font(FontId::proportional(16.0)))
                            } else {
                                ui.button(RichText::new(&btn_text).font(FontId::proportional(16.0)))
                            };

                            if btn.clicked() {
                                if is_listening {
                                    ui_state.listening_for = None;
                                } else {
                                    ui_state.listening_for = Some(act.to_string());
                                }
                            }
                        }

                        if ui_state.listening_for.is_some() {
                            ui.label(RichText::new("Press any key...").font(FontId::proportional(14.0)).color(Color32::from_rgb(255, 200, 0)));
                        }

                        ui.add_space(16.0);
                        ui.label(RichText::new("Display").font(FontId::proportional(24.0)).color(header_color));
                        ui.add_space(4.0);

                        ui.checkbox(&mut settings.vsync, RichText::new("VSync").font(FontId::proportional(16.0)).color(Color32::WHITE));

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("FPS Cap:").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            let fps_label = match settings.fps_cap {
                                Some(cap) => cap.to_string(),
                                None => "Uncapped".to_string(),
                            };
                            egui::ComboBox::from_id_salt("fps_cap")
                                .selected_text(&fps_label)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(settings.fps_cap.is_none(), "Uncapped").clicked() {
                                        settings.fps_cap = None;
                                    }
                                    for &cap in FPS_CAPS {
                                        if ui.selectable_label(settings.fps_cap == Some(cap), cap.to_string()).clicked() {
                                            settings.fps_cap = Some(cap);
                                        }
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Aspect Ratio:").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            let ratio = settings.aspect_ratio.clone();
                            egui::ComboBox::from_id_salt("aspect_ratio")
                                .selected_text(&ratio)
                                .show_ui(ui, |ui| {
                                    for &ar in ASPECT_RATIOS {
                                        if ui.selectable_label(settings.aspect_ratio == ar, ar).clicked() {
                                            settings.aspect_ratio = ar.to_string();
                                            let resolutions = resolutions_for_ratio(ar);
                                            if !resolutions.contains(&settings.resolution) {
                                                settings.resolution = resolutions[0];
                                            }
                                        }
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Resolution:").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            let res_label = resolution_label(settings.resolution);
                            let resolutions = resolutions_for_ratio(&settings.aspect_ratio);
                            egui::ComboBox::from_id_salt("resolution")
                                .selected_text(&res_label)
                                .show_ui(ui, |ui| {
                                    for &res in &resolutions {
                                        let label = resolution_label(res);
                                        if ui.selectable_label(settings.resolution == res, &label).clicked() {
                                            settings.resolution = res;
                                        }
                                    }
                                });
                        });

                        ui.checkbox(&mut settings.fullscreen, RichText::new("Fullscreen").font(FontId::proportional(16.0)).color(Color32::WHITE));

                        ui.add_space(16.0);
                        ui.label(RichText::new("Background").font(FontId::proportional(24.0)).color(header_color));
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Background:").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            let bg_label = match &settings.background_path {
                                Some(path) => {
                                    let file_name = std::path::Path::new(path)
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| path.clone());
                                    file_name
                                }
                                None => "None".to_string(),
                            };
                            ui.label(RichText::new(bg_label).font(FontId::proportional(16.0)).color(Color32::from_rgb(200, 200, 200)));
                        });

                        ui.horizontal(|ui| {
                            if ui.add(
                                egui::Button::new(RichText::new("Choose Custom").font(FontId::proportional(16.0)).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(0, 150, 160))
                                    .min_size(egui::vec2(120.0, 30.0))
                            ).clicked() {
                                ui_state.pick_background = true;
                            }
                            if settings.background_path.is_some() {
                                if ui.add(
                                    egui::Button::new(RichText::new("Clear").font(FontId::proportional(16.0)).color(Color32::WHITE))
                                        .fill(Color32::from_rgb(150, 50, 50))
                                        .min_size(egui::vec2(80.0, 30.0))
                                ).clicked() {
                                    settings.background_path = None;
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Background Dim:").font(FontId::proportional(16.0)).color(Color32::WHITE));
                            ui.add(egui::Slider::new(&mut settings.background_dim, 0.0..=1.0).show_value(false));
                            ui.label(RichText::new(format!("{}%", (settings.background_dim * 100.0) as u32)).font(FontId::proportional(16.0)).color(Color32::from_rgb(200, 200, 200)));
                        });

                        ui.add_space(24.0);

                        let back_btn = ui.add(
                            egui::Button::new(RichText::new("Back").font(FontId::proportional(20.0)).color(Color32::WHITE))
                                .fill(Color32::from_rgb(0, 150, 160))
                                .min_size(egui::vec2(120.0, 40.0))
                        );
                        if back_btn.clicked() {
                            action = SettingsAction::Back;
                        }
                    });
                });
            });
        });
    });

    if ui_state.pick_background {
        ui_state.pick_background = false;
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .set_title("Choose Background Image")
            .pick_file()
        {
            settings.background_path = Some(path.to_string_lossy().to_string());
        }
    }

    egui_winit.handle_platform_output(window, full_output.platform_output);

    let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

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
            return action;
        }
        Err(e) => {
            eprintln!("Surface error: {e}");
            window.request_redraw();
            return action;
        }
    };

    let view = surface_output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("settings encoder"),
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
            label: Some("settings render pass"),
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

    action
}

pub fn handle_settings_key(key: &Key, ui_state: &mut SettingsUiState, settings: &mut Settings) -> bool {
    if let Some(action) = &ui_state.listening_for {
        let key_str = key_to_string(key);
        match action.as_str() {
            "move_left" => settings.key_move_left = key_str,
            "move_right" => settings.key_move_right = key_str,
            "soft_drop" => settings.key_soft_drop = key_str,
            "hard_drop" => settings.key_hard_drop = key_str,
            "rotate_cw" => settings.key_rotate_cw = key_str,
            "rotate_ccw" => settings.key_rotate_ccw = key_str,
            "rotate_180" => settings.key_rotate_180 = key_str,
            "hold" => settings.key_hold = key_str,
            _ => {}
        }
        ui_state.listening_for = None;
        return true;
    }
    false
}
