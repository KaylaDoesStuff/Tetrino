use egui::{Color32, FontId, Pos2, RichText, Stroke};

use crate::logic::graphics::{self, GpuState};
use winit::window::Window;

pub struct JoinState {
    pub code: String,
    pub error_message: Option<String>,
}

impl Default for JoinState {
    fn default() -> Self {
        Self {
            code: String::new(),
            error_message: None,
        }
    }
}

pub enum JoinAction {
    None,
    Back,
    Join(String, String),
}

pub fn render_join(
    ctx: &egui::Context,
    state: &mut JoinState,
    window: &Window,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) -> JoinAction {
    let mut action = JoinAction::None;

    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width as f32;
    let surf_h = gpu.surface_config.height as f32;

    let full_output = ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w, surf_h, dim);

        let cyan = Color32::from_rgb(0, 150, 160);
        let header_color = Color32::from_rgb(0, 200, 220);
        let label_color = Color32::WHITE;
        let dim_color = Color32::from_rgb(150, 150, 150);

        // Back button - top left
        egui::Area::new(egui::Id::new("join_back"))
            .fixed_pos(Pos2::new(20.0, 20.0))
            .interactable(true)
            .show(ctx, |ui| {
                let btn = ui.add(
                    egui::Button::new(
                        RichText::new("Back")
                            .font(FontId::proportional(20.0))
                            .color(label_color),
                    )
                    .fill(Color32::from_rgb(0, 150, 160))
                    .min_size(egui::vec2(120.0, 40.0)),
                );
                if btn.clicked() {
                    action = JoinAction::Back;
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // Title
            let title_galley = ui.painter().layout(
                "Join Room".to_string(),
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
            ui.add_space(60.0);

            // Centered card
            let card_w = (ui.available_width() * 0.4).min(400.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - card_w).max(0.0) / 2.0);
                ui.vertical(|ui| {
                    ui.set_min_width(card_w);
                    egui::Frame::none()
                        .fill(Color32::from_rgba_premultiplied(20, 20, 30, 200))
                        .stroke(Stroke::new(1.0, cyan))
                        .inner_margin(20.0)
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            ui.label(
                                RichText::new("Join Game")
                                    .font(FontId::proportional(20.0))
                                    .color(header_color),
                            );
                            ui.add_space(12.0);

                            ui.label(
                                RichText::new("Room Code")
                                    .font(FontId::proportional(14.0))
                                    .color(dim_color),
                            );
                            let code_response = ui.add(
                                egui::TextEdit::singleline(&mut state.code)
                                    .hint_text("e.g. TR-A7K2M9Q1X3FB")
                                    .desired_width(ui.available_width()),
                            );

                            if let Some(err) = &state.error_message {
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(err)
                                        .font(FontId::proportional(14.0))
                                        .color(Color32::from_rgb(255, 80, 80)),
                                );
                            }

                            ui.add_space(12.0);

                            let can_join = !state.code.trim().is_empty();
                            let join_btn = ui.add_enabled(
                                can_join,
                                egui::Button::new(
                                    RichText::new("Join")
                                        .font(FontId::proportional(18.0))
                                        .color(label_color),
                                )
                                .fill(if can_join {
                                    Color32::from_rgb(0, 150, 80)
                                } else {
                                    Color32::from_rgb(60, 60, 60)
                                })
                                .min_size(egui::vec2(ui.available_width(), 40.0)),
                            );

                            let enter_pressed = code_response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if join_btn.clicked() || enter_pressed {
                                if can_join {
                                    match crate::logic::network::decode_room_code(state.code.trim()) {
                                        Ok((ip, room_code)) => {
                                            action = JoinAction::Join(ip, room_code);
                                        }
                                        Err(e) => {
                                            state.error_message = Some(e);
                                        }
                                    }
                                }
                            }
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
            label: Some("join encoder"),
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
            label: Some("join render pass"),
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
