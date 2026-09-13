use egui::{Color32, FontId, Pos2, RichText, ScrollArea, Stroke};
use rand::Rng;

use crate::logic::graphics::{self, GpuState};
use winit::window::Window;

const ROOM_CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..ROOM_CODE_CHARS.len());
            ROOM_CODE_CHARS[idx] as char
        })
        .collect()
}

pub struct LobbyState {
    pub garbage_enabled: bool,
    pub b2b_enabled: bool,
    pub combo_enabled: bool,
    pub tspin_enabled: bool,
    pub room_code: String,
    pub combined_code: String,
    pub copied_timer: f32,
}

impl Default for LobbyState {
    fn default() -> Self {
        Self {
            garbage_enabled: true,
            b2b_enabled: true,
            combo_enabled: true,
            tspin_enabled: true,
            room_code: generate_room_code(),
            combined_code: String::new(),
            copied_timer: 0.0,
        }
    }
}

pub enum LobbyAction {
    None,
    Back,
    Start,
}

pub fn render_lobby(
    ctx: &egui::Context,
    state: &mut LobbyState,
    window: &Window,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) -> LobbyAction {
    let mut action = LobbyAction::None;

    let raw_input = egui_winit.take_egui_input(window);

    state.copied_timer = (state.copied_timer - 0.016).max(0.0);

    let surf_w = gpu.surface_config.width as f32;
    let surf_h = gpu.surface_config.height as f32;

    let full_output = ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w, surf_h, dim);

        let cyan = Color32::from_rgb(0, 150, 160);
        let header_color = Color32::from_rgb(0, 200, 220);
        let label_color = Color32::WHITE;
        let dim_color = Color32::from_rgb(150, 150, 150);

        // Exit button - top left
        egui::Area::new(egui::Id::new("lobby_exit"))
            .fixed_pos(Pos2::new(20.0, 20.0))
            .interactable(true)
            .show(ctx, |ui| {
                let btn = ui.add(
                    egui::Button::new(
                        RichText::new("Exit")
                            .font(FontId::proportional(20.0))
                            .color(label_color),
                    )
                    .fill(Color32::from_rgb(150, 50, 50))
                    .min_size(egui::vec2(120.0, 40.0)),
                );
                if btn.clicked() {
                    action = LobbyAction::Back;
                }
            });

        // Room code - top center
        let code_w = 220.0;
        let code_x = surf_w / 2.0 - code_w / 2.0;
        egui::Area::new(egui::Id::new("lobby_room_code"))
            .fixed_pos(Pos2::new(code_x, 20.0))
            .interactable(true)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                    .stroke(Stroke::new(1.0, cyan))
                    .rounding(4.0)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_min_width(code_w - 32.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("Room Code")
                                    .font(FontId::proportional(14.0))
                                    .color(dim_color),
                            );
                            let code_response = ui.add(
                                egui::Label::new(
                                    RichText::new(&state.combined_code)
                                        .font(FontId::monospace(24.0))
                                        .color(label_color),
                                )
                                .sense(egui::Sense::click()),
                            );
                            if code_response.clicked() {
                                ctx.copy_text(state.combined_code.clone());
                                state.copied_timer = 1.5;
                            }
                            if state.copied_timer > 0.0 {
                                ui.label(
                                    RichText::new("Copied!")
                                        .font(FontId::proportional(12.0))
                                        .color(Color32::from_rgb(0, 220, 0)),
                                );
                            }
                        });
                    });
            });

        // Start button - bottom right
        let start_x = surf_w - 140.0;
        let start_y = surf_h - 60.0;
        egui::Area::new(egui::Id::new("lobby_start"))
            .fixed_pos(Pos2::new(start_x, start_y))
            .interactable(true)
            .show(ctx, |ui| {
                let btn = ui.add(
                    egui::Button::new(
                        RichText::new("Start")
                            .font(FontId::proportional(20.0))
                            .color(label_color),
                    )
                    .fill(Color32::from_rgb(0, 150, 80))
                    .min_size(egui::vec2(120.0, 40.0)),
                );
                if btn.clicked() {
                    action = LobbyAction::Start;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // Title
            let title_galley = ui.painter().layout(
                "Versus Lobby".to_string(),
                FontId::proportional(40.0),
                label_color,
                ui.available_width(),
            );
            let title_x = (ui.available_width() - title_galley.size().x) / 2.0;
            ui.painter().galley(
                Pos2::new(ui.clip_rect().min.x + title_x, ui.clip_rect().min.y + 10.0),
                title_galley,
                label_color,
            );
            ui.add_space(50.0);

            // 3-column layout
            let total_w = ui.available_width();
            let col_gap = 16.0;
            let left_w = total_w * 0.25;
            let right_w = total_w * 0.25;
            let center_w = total_w - left_w - right_w - col_gap * 2.0;

            ui.horizontal(|ui| {
                // --- Left column: Players ---
                ui.vertical(|ui| {
                    ui.set_min_width(left_w);
                    egui::Frame::none()
                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                        .stroke(Stroke::new(1.0, cyan))
                        .inner_margin(12.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new("Players")
                                    .font(FontId::proportional(24.0))
                                    .color(header_color),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("You")
                                    .font(FontId::proportional(18.0))
                                    .color(label_color),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Waiting for opponent...")
                                    .font(FontId::proportional(16.0))
                                    .color(dim_color),
                            );
                        });
                });

                ui.add_space(col_gap);

                // --- Center column: Match Settings ---
                ui.vertical(|ui| {
                    ui.set_min_width(center_w);
                    egui::Frame::none()
                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                        .stroke(Stroke::new(1.0, cyan))
                        .inner_margin(12.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new("Match Settings")
                                    .font(FontId::proportional(24.0))
                                    .color(header_color),
                            );
                            ui.add_space(8.0);
                            ui.checkbox(
                                &mut state.garbage_enabled,
                                RichText::new("Garbage")
                                    .font(FontId::proportional(16.0))
                                    .color(label_color),
                            );
                            ui.checkbox(
                                &mut state.b2b_enabled,
                                RichText::new("Back-to-Back")
                                    .font(FontId::proportional(16.0))
                                    .color(label_color),
                            );
                            ui.checkbox(
                                &mut state.combo_enabled,
                                RichText::new("Combos")
                                    .font(FontId::proportional(16.0))
                                    .color(label_color),
                            );
                            ui.checkbox(
                                &mut state.tspin_enabled,
                                RichText::new("T-Spins")
                                    .font(FontId::proportional(16.0))
                                    .color(label_color),
                            );
                        });
                });

                ui.add_space(col_gap);

                // --- Right column: Chat ---
                ui.vertical(|ui| {
                    ui.set_min_width(right_w);
                    egui::Frame::none()
                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                        .stroke(Stroke::new(1.0, cyan))
                        .inner_margin(12.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new("Chat")
                                    .font(FontId::proportional(24.0))
                                    .color(header_color),
                            );
                            ui.add_space(8.0);
                            ScrollArea::vertical()
                                .max_height(ui.available_height() - 40.0)
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("No messages yet.")
                                            .font(FontId::proportional(14.0))
                                            .color(dim_color),
                                    );
                                });
                            ui.add_space(4.0);
                            ui.add_enabled(
                                false,
                                egui::TextEdit::singleline(&mut "")
                                    .hint_text("Type a message...")
                                    .desired_width(ui.available_width()),
                            );
                        });
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
            label: Some("lobby encoder"),
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
            label: Some("lobby render pass"),
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
