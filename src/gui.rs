use crate::machine::Machine;
use crate::uistate::UIState;
use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_wgpu::{Renderer, ScreenDescriptor};
use pixels::{PixelsContext, wgpu};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,
    first_run_done: bool,

    // State for the GUI
    pub gui: Gui,
}

/// Example application state. A real application will need a lot more state than this.
pub struct Gui {
    /// Show about window.
    show_about: bool,
    /// Show memory monitor.
    show_memory_monitor: bool,
    /// Show debugger.
    show_debugger: bool,
    /// Memory snapshot for the memory monitor.
    memory_snapshot: Vec<u8>,
    /// The UI state.
    pub ui_state: UIState,
    /// Breakpoint input data.
    breakpoint_input: String,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        debug: bool,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            ViewportId::ROOT,
            event_loop,
            Some(scale_factor),
            Some(max_texture_size),
        );
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new(debug);
        // Warm up the context.
        let _ = egui_ctx.run(egui::RawInput::default(), |_| {});

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            first_run_done: false,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_window_event(window, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window, machine: &mut Machine) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);

        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            self.gui.ui(egui_ctx, machine);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, output.platform_output);

        // Only tessellate if we have a valid frame state.
        self.paint_jobs = self
            .egui_ctx
            .tessellate(output.shapes, self.screen_descriptor.pixels_per_point);

        // Mark that we've successfully completed a real pass.
        self.first_run_done = true;
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // If prepare() hasn't been called yet, paint_jobs will be empty.
        // We should only render if we have something to show.
        if !self.first_run_done || self.paint_jobs.is_empty() {
            return;
        }

        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

impl Gui {
    /// Create a `Gui`.
    fn new(debug: bool) -> Self {
        Self {
            show_about: false,
            show_debugger: debug,
            show_memory_monitor: false,
            memory_snapshot: vec![0; 0x10000],
            ui_state: UIState::new(),
            breakpoint_input: String::new(),
        }
    }

    /// Update the current snapshot from memory.
    fn refresh_memory_snapshot(&mut self, machine: &Machine) {
        for i in 0..=0xFFFF {
            self.memory_snapshot[i as usize] = machine.memory.read8(i);
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context, machine: &mut Machine) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("About...").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    };
                    if ui.button("Quit").clicked() {
                        self.ui_state.exit_requested = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Debug", |ui| {
                    if ui.button("Debugger...").clicked() {
                        self.show_debugger = true;
                        ui.close_menu();
                    }
                    if ui.button("Memory monitor...").clicked() {
                        self.show_memory_monitor = true;
                        ui.close_menu();
                    }
                })
            });
        });

        // About window.
        egui::Window::new("Hello, egui!")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.label("This example demonstrates using egui with pixels.");
                ui.label("Made with 💖 in San Francisco!");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x /= 2.0;
                    ui.label("Learn more about egui at");
                    ui.hyperlink("https://docs.rs/egui");
                });
            });

        // Debugger.
        if self.show_debugger {
            egui::Window::new("💻 CPU Debugger")
                .open(&mut self.show_debugger)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        // Control Buttons
                        ui.horizontal(|ui| {
                            let pause_label = if machine.debug.is_paused() {
                                "▶ Continue"
                            } else {
                                "⏸ Pause"
                            };
                            if ui.button(pause_label).clicked() {
                                machine.debug.toggle_paused();
                            }

                            if ui
                                .add_enabled(
                                    machine.debug.is_paused(),
                                    egui::Button::new("Step Instr (F6)"),
                                )
                                .clicked()
                            {
                                machine.debug.request_step_instruction();
                            }

                            if ui
                                .add_enabled(
                                    machine.debug.is_paused(),
                                    egui::Button::new("Step Line (F7)"),
                                )
                                .clicked()
                            {
                                machine.debug.request_step_scanline();
                            }
                        });

                        ui.separator();

                        // Registers Display
                        ui.columns(2, |cols| {
                            cols[0].monospace(format!("PC: 0x{:04X}", machine.registers.pc));
                            cols[0].monospace(format!("SP: 0x{:04X}", machine.registers.sp));
                            cols[1].monospace(format!("AF: 0x{:04X}", machine.registers.get_af()));
                            cols[1].monospace(format!("BC: 0x{:04X}", machine.registers.get_bc()));
                        });

                        ui.separator();

                        // Breakpoints section.
                        ui.label(format!(
                            "Breakpoints: {}",
                            machine.debug.get_breakpoints_str()
                        ));
                        // Simple input for new breakpoint.
                        ui.horizontal(|ui| {
                            ui.label("Add (Hex):");

                            // Use the field from self instead of a static mut
                            ui.text_edit_singleline(&mut self.breakpoint_input);

                            if ui.button("+").clicked() {
                                if let Ok(addr) = u16::from_str_radix(&self.breakpoint_input, 16) {
                                    machine.debug.add_breakpoint(addr);
                                    self.breakpoint_input.clear(); // Optional: clear after adding
                                }
                            }
                        });
                    });
                });
        }

        // Memory monitor window.
        if self.show_memory_monitor {
            let mut is_open = self.show_memory_monitor;
            egui::Window::new("Memory Monitor")
                .open(&mut is_open)
                .default_size([450.0, 400.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("🔄 Refresh Snapshot").clicked() {
                            self.refresh_memory_snapshot(machine);
                        }
                        ui.label(format!("Showing snapshot from address 0x0000 to 0xFFFF"));
                    });

                    ui.separator();

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                            for i in (0..0x10000).step_by(16) {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{:04X}:", i));

                                    // Constructing one string is often faster than 16 individual labels
                                    let mut hex_line = String::with_capacity(48);
                                    for j in 0..16 {
                                        if let Some(&val) = self.memory_snapshot.get(i + j) {
                                            hex_line.push_str(&format!("{:02X} ", val));
                                        }
                                    }
                                    ui.label(hex_line);
                                });
                            }
                        });
                });
        }
    }
}
