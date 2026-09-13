use egui::{Color32, FontId, Pos2, Stroke, epaint::PathShape};
use winit::window::Window;

use super::graphics::{self, GpuState};

pub enum MenuAction {
    None,
    SinglePlayer,
    Multiplayer,
    Settings,
    Exit,
}

pub struct Button {
    text: &'static str,
    top_left: Pos2,
    width: f32,
    height: f32,
    slant: f32,
}

impl Button {
    fn corners(&self) -> [Pos2; 4] {
        [
            Pos2::new(self.top_left.x + self.slant, self.top_left.y),
            Pos2::new(self.top_left.x + self.width, self.top_left.y),
            Pos2::new(
                self.top_left.x + self.width - self.slant,
                self.top_left.y + self.height,
            ),
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

pub fn handle_menu_click(
    mouse_pos: Pos2,
    buttons: &[Button],
) -> MenuAction {
    for button in buttons {
        if button.contains(mouse_pos) {
            return match button.text {
                "Single Player" => MenuAction::SinglePlayer,
                "Multiplayer" => MenuAction::Multiplayer,
                "Settings" => MenuAction::Settings,
                "Exit" => MenuAction::Exit,
                _ => MenuAction::None,
            };
        }
    }
    MenuAction::None
}

pub fn build_buttons(surf_w: f32, surf_h: f32, sf: f32) -> Vec<Button> {
    let logical_w = surf_w / sf;
    let logical_h = surf_h / sf;

    let btn_w = logical_w / 4.0;
    let btn_h = 50.0;
    let slant = 25.0;
    let gap = 20.0;
    let title_bottom = logical_h / 3.0;
    let start_y = title_bottom + 40.0;
    let start_x = logical_w / 2.0 - btn_w / 2.0;

    vec![
        Button { text: "Single Player", top_left: Pos2::new(start_x, start_y), width: btn_w, height: btn_h, slant },
        Button { text: "Multiplayer",  top_left: Pos2::new(start_x, start_y + btn_h + gap), width: btn_w, height: btn_h, slant },
        Button { text: "Settings", top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 2.0), width: btn_w, height: btn_h, slant },
        Button { text: "Exit", top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 3.0), width: btn_w, height: btn_h, slant },
    ]
}

pub fn render_menu(
    window: &Window,
    egui_ctx: &egui::Context,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) {
    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width;
    let surf_h = gpu.surface_config.height;
    let sf = window.scale_factor() as f32;

    let buttons = build_buttons(surf_w as f32, surf_h as f32, sf);

    let full_output = egui_ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w as f32, surf_h as f32, dim);

        let layer = egui::LayerId::new(egui::Order::Background, egui::Id::new("menu"));
        let painter = ctx.layer_painter(layer);

        let logical_w = surf_w as f32 / sf;
        let logical_h = surf_h as f32 / sf;

        let title_galley = painter.layout(
            "Tetrino".to_string(),
            FontId::proportional(64.0),
            Color32::WHITE,
            logical_w,
        );
        let title_x = (logical_w - title_galley.size().x) / 2.0;
        let title_y = (logical_h / 3.0 - title_galley.size().y) / 2.0;
        painter.galley(
            Pos2::new(title_x, title_y),
            title_galley,
            Color32::WHITE,
        );

        let cyan = Color32::from_rgb(0, 150, 160);
        let dark_cyan = Color32::from_rgb(0, 120, 130);
        let stroke = Stroke::new(2.0_f32, dark_cyan);

        for button in &buttons {
            let corners = button.corners();
            let points: Vec<Pos2> = corners.to_vec();
            let path = PathShape::convex_polygon(points, cyan, stroke);
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
            painter.galley(
                Pos2::new(text_x, text_y),
                text_galley,
                Color32::WHITE,
            );
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
            label: Some("menu encoder"),
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
            label: Some("menu render pass"),
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

pub enum ModeAction {
    None,
    Sprint40L,
    Blitz,
    Versus,
    Join,
    Custom,
    Endless,
    Leaderboards,
    Profile,
}

pub fn build_single_player_buttons(surf_w: f32, surf_h: f32, sf: f32) -> Vec<Button> {
    let logical_w = surf_w / sf;
    let logical_h = surf_h / sf;

    let btn_w = logical_w / 4.0;
    let btn_h = 50.0;
    let slant = 25.0;
    let gap = 20.0;
    let title_bottom = logical_h / 3.0;
    let start_y = title_bottom + 40.0;
    let start_x = logical_w / 2.0 - btn_w / 2.0;

    vec![
        Button { text: "40L",      top_left: Pos2::new(start_x, start_y),                         width: btn_w, height: btn_h, slant },
        Button { text: "Blitz",    top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 1.0),   width: btn_w, height: btn_h, slant },
        Button { text: "Custom",   top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 2.0),   width: btn_w, height: btn_h, slant },
        Button { text: "Endless",  top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 3.0),   width: btn_w, height: btn_h, slant },
    ]
}

pub fn handle_single_player_click(mouse_pos: Pos2, buttons: &[Button]) -> ModeAction {
    for button in buttons {
        if button.contains(mouse_pos) {
            return match button.text {
                "40L" => ModeAction::Sprint40L,
                "Blitz" => ModeAction::Blitz,
                "Custom" => ModeAction::Custom,
                "Endless" => ModeAction::Endless,
                _ => ModeAction::None,
            };
        }
    }
    ModeAction::None
}

pub fn build_multiplayer_buttons(surf_w: f32, surf_h: f32, sf: f32) -> Vec<Button> {
    let logical_w = surf_w / sf;
    let logical_h = surf_h / sf;

    let btn_w = logical_w / 4.0;
    let btn_h = 50.0;
    let slant = 25.0;
    let gap = 20.0;
    let title_bottom = logical_h / 3.0;
    let start_y = title_bottom + 40.0;
    let start_x = logical_w / 2.0 - btn_w / 2.0;

    vec![
        Button { text: "Versus",      top_left: Pos2::new(start_x, start_y),                         width: btn_w, height: btn_h, slant },
        Button { text: "Join",        top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 1.0),  width: btn_w, height: btn_h, slant },
        Button { text: "Leaderboards", top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 2.0),  width: btn_w, height: btn_h, slant },
        Button { text: "Profile",      top_left: Pos2::new(start_x, start_y + (btn_h + gap) * 3.0),  width: btn_w, height: btn_h, slant },
    ]
}

pub fn handle_multiplayer_click(mouse_pos: Pos2, buttons: &[Button]) -> ModeAction {
    for button in buttons {
        if button.contains(mouse_pos) {
            return match button.text {
                "Versus" => ModeAction::Versus,
                "Join" => ModeAction::Join,
                "Leaderboards" => ModeAction::Leaderboards,
                "Profile" => ModeAction::Profile,
                _ => ModeAction::None,
            };
        }
    }
    ModeAction::None
}

pub fn render_single_player_mode_select(
    window: &Window,
    egui_ctx: &egui::Context,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) {
    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width;
    let surf_h = gpu.surface_config.height;
    let sf = window.scale_factor() as f32;

    let buttons = build_single_player_buttons(surf_w as f32, surf_h as f32, sf);

    let full_output = egui_ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w as f32, surf_h as f32, dim);

        let logical_w = surf_w as f32 / sf;
        let logical_h = surf_h as f32 / sf;

        let layer = egui::LayerId::new(egui::Order::Background, egui::Id::new("modeselect"));
        let painter = ctx.layer_painter(layer);

        let title_galley = painter.layout(
            "Select Mode".to_string(),
            FontId::proportional(40.0),
            Color32::WHITE,
            logical_w,
        );
        let title_x = (logical_w - title_galley.size().x) / 2.0;
        let title_y = (logical_h / 3.0 - title_galley.size().y) / 2.0;
        painter.galley(
            Pos2::new(title_x, title_y),
            title_galley,
            Color32::WHITE,
        );

        let cyan = Color32::from_rgb(0, 150, 160);
        let dark_cyan = Color32::from_rgb(0, 120, 130);
        let stroke = Stroke::new(2.0_f32, dark_cyan);

        for button in &buttons {
            let corners = button.corners();
            let points: Vec<Pos2> = corners.to_vec();
            let path = PathShape::convex_polygon(points, cyan, stroke);
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
            painter.galley(
                Pos2::new(text_x, text_y),
                text_galley,
                Color32::WHITE,
            );
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
            label: Some("modeselect encoder"),
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
            label: Some("modeselect render pass"),
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

pub fn render_multiplayer_mode_select(
    window: &Window,
    egui_ctx: &egui::Context,
    egui_winit: &mut egui_winit::State,
    gpu: &mut GpuState,
    background_texture: &Option<egui::TextureHandle>,
    dim: f32,
) {
    let raw_input = egui_winit.take_egui_input(window);

    let surf_w = gpu.surface_config.width;
    let surf_h = gpu.surface_config.height;
    let sf = window.scale_factor() as f32;

    let buttons = build_multiplayer_buttons(surf_w as f32, surf_h as f32, sf);

    let full_output = egui_ctx.run(raw_input, |ctx| {
        graphics::render_background(ctx, background_texture, surf_w as f32, surf_h as f32, dim);

        let logical_w = surf_w as f32 / sf;
        let logical_h = surf_h as f32 / sf;

        let layer = egui::LayerId::new(egui::Order::Background, egui::Id::new("multiplayermodeselect"));
        let painter = ctx.layer_painter(layer);

        let title_galley = painter.layout(
            "Multiplayer".to_string(),
            FontId::proportional(40.0),
            Color32::WHITE,
            logical_w,
        );
        let title_x = (logical_w - title_galley.size().x) / 2.0;
        let title_y = (logical_h / 3.0 - title_galley.size().y) / 2.0;
        painter.galley(
            Pos2::new(title_x, title_y),
            title_galley,
            Color32::WHITE,
        );

        let cyan = Color32::from_rgb(0, 150, 160);
        let dark_cyan = Color32::from_rgb(0, 120, 130);
        let stroke = Stroke::new(2.0_f32, dark_cyan);

        for button in &buttons {
            let corners = button.corners();
            let points: Vec<Pos2> = corners.to_vec();
            let path = PathShape::convex_polygon(points, cyan, stroke);
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
            painter.galley(
                Pos2::new(text_x, text_y),
                text_galley,
                Color32::WHITE,
            );
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
            label: Some("multiplayermodeselect encoder"),
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
            label: Some("multiplayermodeselect render pass"),
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
