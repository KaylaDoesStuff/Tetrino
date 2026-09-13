use egui::{Color32, FontId, Pos2, RichText, ScrollArea, Stroke};
use winit::window::Window;

use super::graphics::{self, GpuState};

pub enum ProfileAction {
    None,
    Back,
}

pub struct RatingStats {
    pub elo: u32,
    pub games_played: u32,
    pub games_won: u32,
    pub avg_finesse_pct: f32,
    pub avg_pps: f32,
    pub avg_vs: f32,
    pub avg_apm: f32,
}

pub struct ModeStats {
    pub best: String,
    pub finesse_pct: f32,
    pub faults: u32,
    pub pps: f32,
    pub lpm: f32,
    pub games_played: u32,
}

pub struct ProfileData {
    pub username: String,
    pub level: u32,
    pub rating: RatingStats,
    pub sprint_40l: ModeStats,
    pub blitz: ModeStats,
}

impl ProfileData {
    pub fn placeholder() -> Self {
        Self {
            username: "Player".to_string(),
            level: 1,
            rating: RatingStats {
                elo: 1500,
                games_played: 0,
                games_won: 0,
                avg_finesse_pct: 0.0,
                avg_pps: 0.0,
                avg_vs: 0.0,
                avg_apm: 0.0,
            },
            sprint_40l: ModeStats {
                best: "--:--.---".to_string(),
                finesse_pct: 0.0,
                faults: 0,
                pps: 0.0,
                lpm: 0.0,
                games_played: 0,
            },
            blitz: ModeStats {
                best: "0".to_string(),
                finesse_pct: 0.0,
                faults: 0,
                pps: 0.0,
                lpm: 0.0,
                games_played: 0,
            },
        }
    }
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{label}: "))
                .font(FontId::proportional(16.0))
                .color(Color32::from_rgb(160, 160, 160)),
        );
        ui.label(
            RichText::new(value)
                .font(FontId::proportional(16.0))
                .color(color),
        );
    });
}

pub fn render_profile(
    ctx: &egui::Context,
    profile: &ProfileData,
    window: &Window,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) -> ProfileAction {
    let mut action = ProfileAction::None;

    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width as f32;
    let surf_h = gpu.surface_config.height as f32;

    let full_output = ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w, surf_h, dim);

        egui::Area::new(egui::Id::new("profile_back"))
            .fixed_pos(Pos2::new(20.0, 20.0))
            .interactable(true)
            .show(ctx, |ui| {
                let btn = ui.add(
                    egui::Button::new(
                        RichText::new("Back")
                            .font(FontId::proportional(20.0))
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(0, 150, 160))
                    .min_size(egui::vec2(120.0, 40.0)),
                );
                if btn.clicked() {
                    action = ProfileAction::Back;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            let header_galley = ui.painter().layout(
                "Profile".to_string(),
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
                let header_color = Color32::from_rgb(0, 200, 220);
                let value_color = Color32::WHITE;

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&profile.username)
                                    .font(FontId::proportional(28.0))
                                    .color(Color32::WHITE),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("Level: {}", profile.level))
                                        .font(FontId::proportional(20.0))
                                        .color(Color32::from_rgb(200, 200, 255)),
                                );
                            });
                        });
                        ui.add_space(16.0);

                        // --- Rating ---
                        egui::Frame::none()
                            .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0, 150, 160)))
                            .inner_margin(12.0)
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    RichText::new("Rating")
                                        .font(FontId::proportional(24.0))
                                        .color(header_color),
                                );
                                ui.add_space(4.0);
                                stat_row(ui, "ELO", &profile.rating.elo.to_string(), value_color);
                                stat_row(ui, "Games Played", &profile.rating.games_played.to_string(), value_color);
                                stat_row(ui, "Games Won", &profile.rating.games_won.to_string(), value_color);
                                stat_row(
                                    ui,
                                    "Avg Finesse",
                                    &format!("{:.1}%", profile.rating.avg_finesse_pct),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Avg PPS",
                                    &format!("{:.2}", profile.rating.avg_pps),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Avg VS",
                                    &format!("{:.1}", profile.rating.avg_vs),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Avg APM",
                                    &format!("{:.1}", profile.rating.avg_apm),
                                    value_color,
                                );
                            });
                        ui.add_space(20.0);

                        // --- 40 Lines ---
                        egui::Frame::none()
                            .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0, 150, 160)))
                            .inner_margin(12.0)
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    RichText::new("40 Lines")
                                        .font(FontId::proportional(24.0))
                                        .color(header_color),
                                );
                                ui.add_space(4.0);
                                stat_row(ui, "Best", &profile.sprint_40l.best, value_color);
                                stat_row(
                                    ui,
                                    "Finesse",
                                    &format!("{:.1}%", profile.sprint_40l.finesse_pct),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Faults",
                                    &profile.sprint_40l.faults.to_string(),
                                    if profile.sprint_40l.faults > 0 {
                                        Color32::from_rgb(255, 100, 100)
                                    } else {
                                        value_color
                                    },
                                );
                                stat_row(
                                    ui,
                                    "PPS",
                                    &format!("{:.2}", profile.sprint_40l.pps),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "LPM",
                                    &format!("{:.1}", profile.sprint_40l.lpm),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Games Played",
                                    &profile.sprint_40l.games_played.to_string(),
                                    value_color,
                                );
                            });
                        ui.add_space(20.0);

                        // --- Blitz ---
                        egui::Frame::none()
                            .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0, 150, 160)))
                            .inner_margin(12.0)
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    RichText::new("Blitz")
                                        .font(FontId::proportional(24.0))
                                        .color(header_color),
                                );
                                ui.add_space(4.0);
                                stat_row(ui, "Best", &profile.blitz.best, value_color);
                                stat_row(
                                    ui,
                                    "Finesse",
                                    &format!("{:.1}%", profile.blitz.finesse_pct),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Faults",
                                    &profile.blitz.faults.to_string(),
                                    if profile.blitz.faults > 0 {
                                        Color32::from_rgb(255, 100, 100)
                                    } else {
                                        value_color
                                    },
                                );
                                stat_row(
                                    ui,
                                    "PPS",
                                    &format!("{:.2}", profile.blitz.pps),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "LPM",
                                    &format!("{:.1}", profile.blitz.lpm),
                                    value_color,
                                );
                                stat_row(
                                    ui,
                                    "Games Played",
                                    &profile.blitz.games_played.to_string(),
                                    value_color,
                                );
                            });
            });
        });
    });

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
            label: Some("profile encoder"),
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
            label: Some("profile render pass"),
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
