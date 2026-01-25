use crate::constants;
use crate::instruction::RunInstr;
use crate::machine::Machine;
use crate::uistate::UIState;
use egui::{
    ClippedPrimitive, CollapsingHeader, Color32, Context, FontFamily, FontId, RichText, ScrollArea,
    TexturesDelta, ViewportId, text::LayoutJob, vec2,
};
use egui_wgpu::{Renderer, ScreenDescriptor};
use pixels::{PixelsContext, wgpu};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

pub const BLUE: Color32 = Color32::from_rgb(66, 133, 244);
pub const GRAY: Color32 = Color32::from_rgb(127, 127, 127);
pub const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
pub const CYAN: Color32 = Color32::from_rgb(0, 188, 212);
pub const MAGENTA: Color32 = Color32::from_rgb(233, 30, 99);
pub const YELLOW: Color32 = Color32::from_rgb(244, 180, 0);
pub const GREEN: Color32 = Color32::from_rgb(15, 157, 88);
pub const RED: Color32 = Color32::from_rgb(219, 68, 55);
pub const ORANGE: Color32 = Color32::from_rgb(255, 152, 0);

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    pub egui_ctx: Context,
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
    /// Show debugger.
    show_debugger: bool,
    /// Show FPS.
    show_fps: bool,
    /// The menu timer.
    menu_timer: f32,
    /// Last mouse position.
    last_mouse_pos: Option<egui::Pos2>,
    /// The UI state.
    pub ui_state: UIState,
    /// Breakpoint input data.
    breakpoint_input: String,
    /// Input error in breakpoints.
    breakpoint_error: bool,
    /// Logo texture.
    logo_texture: Option<egui::TextureHandle>,
    /// Last PC.
    last_pc: u16,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
        fps: bool,
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
        let gui = Gui::new(debug, fps);
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
    fn new(show_debugger: bool, show_fps: bool) -> Self {
        Self {
            show_about: false,
            show_debugger,
            show_fps,
            menu_timer: 0.0,
            last_mouse_pos: None,
            ui_state: UIState::new(),
            breakpoint_input: String::new(),
            breakpoint_error: false,
            logo_texture: None,
            last_pc: 0,
        }
    }

    /// Toggle state of FPS.
    pub fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
    }

    pub fn show_debugger(&mut self, show: bool) {
        self.show_debugger = show;
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context, machine: &mut Machine) {
        let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
        let dt = ctx.input(|i| i.stable_dt);

        // Check for movement.
        let mouse_moved = if let (Some(current), Some(last)) = (mouse_pos, self.last_mouse_pos) {
            current != last
        } else {
            false
        };
        // Check sensors.
        let mouse_at_top = mouse_pos.map_or(false, |pos| pos.y < 30.0);
        let menu_in_use = ctx.input(|i| i.pointer.has_pointer())
            && ctx
                .layer_id_at(mouse_pos.unwrap_or_default())
                .map_or(false, |l| {
                    l.order == egui::Order::Foreground || l.order == egui::Order::Tooltip
                });

        // Update Timer (5s).
        if mouse_moved || mouse_at_top || menu_in_use {
            self.menu_timer = 5.0;
        } else {
            self.menu_timer -= dt;
        }
        self.last_mouse_pos = mouse_pos;

        if self.menu_timer > 0.0 {
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
                    ui.menu_button("Graphics", |ui| {
                        ui.menu_button("Palette", |ui| {
                            let current_palette = machine.memory.ppu.get_palette_index();

                            for (i, name) in crate::ppu::PALETTE_NAMES.iter().enumerate() {
                                let i = i as u8;
                                if ui.radio(current_palette == i, *name).clicked() {
                                    machine.memory.ppu.set_palette(i);
                                }
                            }
                        });
                        if ui.button("Save screenshot").clicked() {
                            self.ui_state.screenshot_requested = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Machine", |ui| {
                        if ui.button("Reset CPU").clicked() {
                            machine.reset();
                            ui.close_menu();
                        }
                        if ui.checkbox(&mut self.show_fps, "Show FPS").clicked() {
                            ui.close_menu();
                        }
                        if ui.button("Debug panel...").clicked() {
                            self.show_debugger = true;
                            ui.close_menu();
                        };
                    })
                });
            });
        }

        // About window.
        egui::Window::new(constants::NAME)
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                let texture: &egui::TextureHandle = self.logo_texture.get_or_insert_with(|| {
                    // Embed the file at compile time.
                    let image_data = include_bytes!("../img/logo.png");
                    let image = image::load_from_memory(image_data)
                        .expect("Failed to load logo")
                        .to_rgba8();

                    let (width, height) = image.dimensions();
                    let size = [width as usize, height as usize];
                    let pixels = image.as_flat_samples();

                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                    ctx.load_texture("app-logo", color_image, egui::TextureOptions::NEAREST)
                });
                ui.vertical_centered(|ui| {
                    // Display the logo scaled up.
                    ui.image((texture.id(), texture.size_vec2()));
                    ui.heading(env!("CARGO_PKG_VERSION"));
                    ui.add_space(10.0); // Padding

                    ui.label(env!("CARGO_PKG_DESCRIPTION"));

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(env!("CARGO_PKG_AUTHORS"));
                    ui.hyperlink(env!("CARGO_PKG_HOMEPAGE"));
                    ui.hyperlink(env!("CARGO_PKG_REPOSITORY"));

                    ui.add_space(10.0);
                });
            });

        // Debugger.
        if self.show_debugger {
            egui::Window::new("💻 CPU Debugger")
                .open(&mut self.show_debugger)
                .show(ctx, |ui| {
                    let pc = machine.registers.pc;
                    let opcode = machine.memory.read8(pc);
                    ui.vertical(|ui| {
                        // Control Buttons.
                        ui.horizontal_top(|ui| {
                            let button_size = egui::vec2(120.0, 0.0);

                            // Step instruction.
                            let mut stepi_label = LayoutJob::default();
                            RichText::new("⤴ Step Instr")
                                .color(BLUE)
                                .font(FontId::new(14.0, FontFamily::Proportional))
                                .strong()
                                .append_to(
                                    &mut stepi_label,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            RichText::new("   [F6]")
                                .color(GRAY)
                                .font(FontId::new(10.0, FontFamily::Proportional))
                                .weak()
                                .append_to(
                                    &mut stepi_label,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            if ui
                                .add_enabled_ui(machine.debug.is_paused(), |ui| {
                                    ui.add_sized(button_size, egui::Button::new(stepi_label))
                                        .on_hover_text("Step one instruction. [F6]")
                                })
                                .inner
                                .clicked()
                            {
                                machine.debug.request_step_instruction();
                            }

                            // Step scanline.
                            let mut steps_label = LayoutJob::default();
                            RichText::new("⮫ Step Line")
                                .color(BLUE)
                                .font(FontId::new(14.0, FontFamily::Proportional))
                                .strong()
                                .append_to(
                                    &mut steps_label,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            RichText::new("   [F7]")
                                .color(GRAY)
                                .font(FontId::new(10.0, FontFamily::Proportional))
                                .weak()
                                .append_to(
                                    &mut steps_label,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            if ui
                                .add_enabled_ui(machine.debug.is_paused(), |ui| {
                                    ui.add_sized(button_size, egui::Button::new(steps_label))
                                        .on_hover_text("Step a scanline. [F7]")
                                })
                                .inner
                                .clicked()
                            {
                                machine.debug.request_step_scanline();
                            }

                            // Continue/Pause
                            let pause_text = if machine.debug.is_paused() {
                                "▶ Continue"
                            } else {
                                "⏸ Pause"
                            };
                            let mut pause_label = LayoutJob::default();
                            RichText::new(pause_text)
                                .color(GREEN)
                                .font(FontId::new(14.0, FontFamily::Proportional))
                                .strong()
                                .append_to(
                                    &mut pause_label,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            RichText::new("   [F9]")
                                .color(GRAY)
                                .font(FontId::new(10.0, FontFamily::Proportional))
                                .weak()
                                .append_to(
                                    &mut pause_label,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                            if ui
                                .add_sized(button_size, egui::Button::new(pause_label))
                                .on_hover_text("Continue/pause. [F9]")
                                .clicked()
                            {
                                machine.debug.toggle_paused();
                            }
                        });

                        ui.separator();
                        ui.add_space(8.0);

                        // Main horizontal pane.
                        ui.columns(2, |columns| {
                            // LEFT: Instruction, CPU, PPU, JOYP, BREAKP.
                            columns[0].vertical(|ui| {
                                ui.allocate_space(vec2(320.0, 0.0));
                                // Current instruction.
                                let run_instr =
                                    RunInstr::new(opcode, &machine.memory, &machine.registers);
                                let mut instr = LayoutJob::default();
                                RichText::new(format!("${:04x}:", pc))
                                    .color(GRAY)
                                    .font(FontId::new(18.0, FontFamily::Monospace))
                                    .strong()
                                    .append_to(
                                        &mut instr,
                                        ui.style(),
                                        egui::FontSelection::Default,
                                        egui::Align::Center,
                                    );
                                RichText::new(format!(" {}", run_instr.instruction_str()))
                                    .color(ORANGE)
                                    .font(FontId::new(18.0, FontFamily::Monospace))
                                    .strong()
                                    .append_to(
                                        &mut instr,
                                        ui.style(),
                                        egui::FontSelection::Default,
                                        egui::Align::Center,
                                    );
                                RichText::new(format!("  {}", run_instr.operand_str()))
                                    .color(WHITE)
                                    .font(FontId::new(18.0, FontFamily::Monospace))
                                    .strong()
                                    .append_to(
                                        &mut instr,
                                        ui.style(),
                                        egui::FontSelection::Default,
                                        egui::Align::Center,
                                    );
                                ui.label(instr)
                                    .on_hover_text("Current PC address, instruction, and operand.");

                                ui.add_space(8.0);

                                CollapsingHeader::new("CPU")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.add_space(8.0);
                                            // Registers Display.
                                            egui::Grid::new("registers_grid")
                                                .num_columns(2)
                                                .min_col_width(100.0)
                                                .spacing([10.0, 4.0])
                                                .show(ui, |ui| {
                                                    // CPU state.
                                                    let mut state = LayoutJob::default();
                                                    RichText::new("State:")
                                                        .color(GRAY)
                                                        .font(FontId::new(
                                                            12.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .strong()
                                                        .append_to(
                                                            &mut state,
                                                            ui.style(),
                                                            egui::FontSelection::Default,
                                                            egui::Align::Center,
                                                        );
                                                    let mut state_val = LayoutJob::default();
                                                    RichText::new(format!(
                                                        "{}",
                                                        if machine.halted {
                                                            "HALTED"
                                                        } else {
                                                            "RUNNING"
                                                        }
                                                    ))
                                                    .color(if machine.halted { RED } else { GREEN })
                                                    .font(FontId::new(16.0, FontFamily::Monospace))
                                                    .strong()
                                                    .append_to(
                                                        &mut state_val,
                                                        ui.style(),
                                                        egui::FontSelection::Default,
                                                        egui::Align::Center,
                                                    );
                                                    ui.label(state);
                                                    ui.label(state_val);
                                                    ui.end_row();

                                                    // Timing Stats
                                                    ui.monospace("T-cycles:");
                                                    ui.monospace(format!("{}", machine.t_cycles));
                                                    ui.end_row();

                                                    ui.monospace("M-cycles:");
                                                    ui.monospace(format!("{}", machine.m_cycles));
                                                    ui.end_row();

                                                    // Registers
                                                    ui.monospace("Registers: ");
                                                    ui.vertical(|ui| {
                                                        // AF.
                                                        let mut af = LayoutJob::default();
                                                        RichText::new("AF ")
                                                            .color(WHITE)
                                                            .font(FontId::new(
                                                                12.0,
                                                                FontFamily::Monospace,
                                                            ))
                                                            .strong()
                                                            .append_to(
                                                                &mut af,
                                                                ui.style(),
                                                                egui::FontSelection::Default,
                                                                egui::Align::Center,
                                                            );
                                                        RichText::new(format!(
                                                            "{:02x} {:02x}",
                                                            machine.registers.a,
                                                            machine.registers.f
                                                        ))
                                                        .color(MAGENTA)
                                                        .font(FontId::new(
                                                            12.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .strong()
                                                        .append_to(
                                                            &mut af,
                                                            ui.style(),
                                                            egui::FontSelection::Default,
                                                            egui::Align::Center,
                                                        );
                                                        ui.label(af);
                                                        // BC.
                                                        let mut bc = LayoutJob::default();
                                                        RichText::new("BC ")
                                                            .color(WHITE)
                                                            .font(FontId::new(
                                                                12.0,
                                                                FontFamily::Monospace,
                                                            ))
                                                            .strong()
                                                            .append_to(
                                                                &mut bc,
                                                                ui.style(),
                                                                egui::FontSelection::Default,
                                                                egui::Align::Center,
                                                            );
                                                        RichText::new(format!(
                                                            "{:02x} {:02x}",
                                                            machine.registers.b,
                                                            machine.registers.c
                                                        ))
                                                        .color(MAGENTA)
                                                        .font(FontId::new(
                                                            12.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .strong()
                                                        .append_to(
                                                            &mut bc,
                                                            ui.style(),
                                                            egui::FontSelection::Default,
                                                            egui::Align::Center,
                                                        );
                                                        ui.label(bc);
                                                        // DE.
                                                        let mut de = LayoutJob::default();
                                                        RichText::new("DE ")
                                                            .color(WHITE)
                                                            .font(FontId::new(
                                                                12.0,
                                                                FontFamily::Monospace,
                                                            ))
                                                            .strong()
                                                            .append_to(
                                                                &mut de,
                                                                ui.style(),
                                                                egui::FontSelection::Default,
                                                                egui::Align::Center,
                                                            );
                                                        RichText::new(format!(
                                                            "{:02x} {:02x}",
                                                            machine.registers.d,
                                                            machine.registers.e
                                                        ))
                                                        .color(MAGENTA)
                                                        .font(FontId::new(
                                                            12.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .strong()
                                                        .append_to(
                                                            &mut de,
                                                            ui.style(),
                                                            egui::FontSelection::Default,
                                                            egui::Align::Center,
                                                        );
                                                        ui.label(de);
                                                        // HL.
                                                        let mut hl = LayoutJob::default();
                                                        RichText::new("HL ")
                                                            .color(WHITE)
                                                            .font(FontId::new(
                                                                12.0,
                                                                FontFamily::Monospace,
                                                            ))
                                                            .strong()
                                                            .append_to(
                                                                &mut hl,
                                                                ui.style(),
                                                                egui::FontSelection::Default,
                                                                egui::Align::Center,
                                                            );
                                                        RichText::new(format!(
                                                            "{:02x} {:02x}",
                                                            machine.registers.h,
                                                            machine.registers.l
                                                        ))
                                                        .color(MAGENTA)
                                                        .font(FontId::new(
                                                            12.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .strong()
                                                        .append_to(
                                                            &mut hl,
                                                            ui.style(),
                                                            egui::FontSelection::Default,
                                                            egui::Align::Center,
                                                        );
                                                        ui.label(hl);
                                                    });
                                                    ui.end_row();

                                                    // Flags
                                                    ui.monospace("Flags:");
                                                    let f = machine.registers.f;
                                                    // Format: Z N H C (matches standard GB nomenclature)
                                                    let z = if f & 0x80 != 0 { "Z" } else { "_" };
                                                    let n = if f & 0x40 != 0 { "N" } else { "_" };
                                                    let h = if f & 0x20 != 0 { "H" } else { "_" };
                                                    let c = if f & 0x10 != 0 { "C" } else { "_" };

                                                    let mut flags = LayoutJob::default();
                                                    RichText::new(format!(
                                                        "{} {} {} {}",
                                                        z, n, h, c
                                                    ))
                                                    .color(CYAN)
                                                    .font(FontId::new(12.0, FontFamily::Monospace))
                                                    .strong()
                                                    .append_to(
                                                        &mut flags,
                                                        ui.style(),
                                                        egui::FontSelection::Default,
                                                        egui::Align::Center,
                                                    );
                                                    ui.label(flags);

                                                    let mem = &machine.memory;

                                                    ui.end_row();
                                                    ui.monospace("Opcode:");
                                                    ui.monospace(format!("{:#02x}", opcode));
                                                    ui.end_row();
                                                    ui.monospace("SP:");
                                                    ui.monospace(format!(
                                                        "{:#04x}",
                                                        machine.registers.sp
                                                    ));
                                                    ui.end_row();
                                                    ui.monospace("DIV:");
                                                    ui.monospace(format!(
                                                        "{:#06x}",
                                                        mem.timer.div16()
                                                    ));
                                                    ui.end_row();
                                                });
                                        });
                                    });

                                CollapsingHeader::new("PPU")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        egui::Grid::new("ppu_grid")
                                            .num_columns(2)
                                            .min_col_width(100.0)
                                            .spacing([10.0, 4.0])
                                            .show(ui, |ui| {
                                                let mem = &machine.memory;
                                                ui.monospace("LCDC:");
                                                ui.monospace(format!("{:#02x}", mem.ppu.lcdc));
                                                ui.end_row();
                                                ui.monospace("STAT:");
                                                ui.monospace(format!("{:#02x}", mem.ppu.stat));
                                                ui.end_row();
                                                ui.monospace("LYC:");
                                                ui.monospace(format!("{:#02x}", mem.ppu.lyc));
                                                ui.end_row();
                                                ui.monospace("LY:");
                                                ui.monospace(format!("{:#02x}", mem.ppu.ly));
                                                ui.end_row();
                                                ui.monospace("LX:");
                                                ui.monospace(format!("{:#02x}", mem.ppu.lx));
                                                ui.end_row();
                                            });
                                    });

                                ui.separator();

                                // JOYPAD.
                                ui.monospace("Joypad:");

                                let mem = &machine.memory;
                                let mut joypad = LayoutJob::default();
                                RichText::new(&format!(
                                    "{} {} {} {} {} {} {} {}",
                                    if mem.joypad.up { "↑" } else { "_" },
                                    if mem.joypad.down { "↓" } else { "_" },
                                    if mem.joypad.left { "←" } else { "_" },
                                    if mem.joypad.right { "→" } else { "_" },
                                    if mem.joypad.a { "A" } else { "_" },
                                    if mem.joypad.b { "B" } else { "_" },
                                    if mem.joypad.start { "S" } else { "_" },
                                    if mem.joypad.select { "s" } else { "_" }
                                ))
                                .color(CYAN)
                                .font(FontId::new(12.0, FontFamily::Monospace))
                                .strong()
                                .append_to(
                                    &mut joypad,
                                    ui.style(),
                                    egui::FontSelection::Default,
                                    egui::Align::Center,
                                );
                                ui.label(joypad);
                                ui.separator();

                                // Breakpoints section.
                                ui.horizontal(|ui| {
                                    ui.label("Breakpoints:");
                                    ui.visuals_mut().override_text_color = Some(YELLOW);
                                    ui.label(format!("{}", machine.debug.get_breakpoints_str()));
                                    ui.visuals_mut().override_text_color = None;
                                });
                                // Simple input for new breakpoint.
                                ui.horizontal(|ui| {
                                    ui.label("Add (Hex):");

                                    if self.breakpoint_error {
                                        ui.visuals_mut().override_text_color = Some(RED);
                                    }
                                    let br_input =
                                        egui::TextEdit::singleline(&mut self.breakpoint_input)
                                            .hint_text("$0123")
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(60.0);
                                    let response = ui.add(br_input);
                                    if response.changed() {
                                        self.breakpoint_error = false;
                                    }
                                    ui.visuals_mut().override_text_color = None;

                                    if ui.button("+").clicked() {
                                        let text = &self
                                            .breakpoint_input
                                            .strip_prefix("$")
                                            .unwrap_or(&self.breakpoint_input);
                                        if let Ok(addr) = u16::from_str_radix(text, 16) {
                                            machine.debug.add_breakpoint(addr);
                                            self.breakpoint_error = false;
                                        } else {
                                            self.breakpoint_error = true;
                                        }
                                    }
                                    if ui.button("-").clicked() {
                                        let text = &self
                                            .breakpoint_input
                                            .strip_prefix("$")
                                            .unwrap_or(&self.breakpoint_input);

                                        if let Ok(addr) = u16::from_str_radix(text, 16) {
                                            machine.debug.delete_breakpoint(addr);
                                        }
                                    }
                                    if ui.button("Clear all").clicked() {
                                        machine.debug.clear_breakpoints();
                                    }
                                });
                            });
                            // RIGHT: Disassembly.
                            columns[1].vertical(|ui| {
                                ui.allocate_space(vec2(290.0, 0.0));
                                ui.heading("Disassembly");
                                ui.separator();

                                // Get the EXACT spacing egui uses between labels.
                                let spacing_y = ui.spacing().item_spacing.y;
                                let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                                // The "True" height of one line entry.
                                let line_height = row_height + spacing_y;

                                let current_pc = machine.registers.pc;
                                let pc_moved = current_pc != self.last_pc;

                                let mut scroll_area =
                                    ScrollArea::vertical().auto_shrink([false; 2]);

                                if pc_moved {
                                    // We calculate the offset based on the spacing-adjusted height.
                                    let target_y = current_pc as f32 * line_height;

                                    let center_offset = target_y - (ui.available_height() / 2.0);

                                    scroll_area =
                                        scroll_area.vertical_scroll_offset(center_offset.max(0.0));
                                    self.last_pc = current_pc;
                                }

                                scroll_area.show_rows(ui, row_height, 0xFFFF, |ui, row_range| {
                                    for addr in row_range {
                                        let is_current_pc = addr == current_pc as usize;

                                        let opcode = machine.memory.read8(addr as u16);
                                        let i = RunInstr::new(
                                            opcode,
                                            &machine.memory,
                                            &machine.registers,
                                        );

                                        let mut instruction = LayoutJob::default();
                                        RichText::new(format!("{:#06x} ", addr))
                                            .color(if machine.debug.has_breakpoint(addr as u16) {
                                                RED
                                            } else {
                                                WHITE
                                            })
                                            .font(FontId::new(12.0, FontFamily::Monospace))
                                            .strong()
                                            .append_to(
                                                &mut instruction,
                                                ui.style(),
                                                egui::FontSelection::Default,
                                                egui::Align::Center,
                                            );
                                        RichText::new(format!("{:#04x} ", opcode))
                                            .color(GRAY)
                                            .font(FontId::new(12.0, FontFamily::Monospace))
                                            .strong()
                                            .append_to(
                                                &mut instruction,
                                                ui.style(),
                                                egui::FontSelection::Default,
                                                egui::Align::Center,
                                            );
                                        RichText::new(format!("{:<4} ", i.instruction_str()))
                                            .color(YELLOW)
                                            .font(FontId::new(12.0, FontFamily::Monospace))
                                            .strong()
                                            .append_to(
                                                &mut instruction,
                                                ui.style(),
                                                egui::FontSelection::Default,
                                                egui::Align::Center,
                                            );
                                        RichText::new(format!("{}", i.operand_str()))
                                            .color(BLUE)
                                            .font(FontId::new(12.0, FontFamily::Monospace))
                                            .strong()
                                            .append_to(
                                                &mut instruction,
                                                ui.style(),
                                                egui::FontSelection::Default,
                                                egui::Align::Center,
                                            );
                                        let galley = ui.fonts(|f| f.layout_job(instruction));
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(ui.available_width(), row_height),
                                            egui::Sense::click(),
                                        );
                                        if is_current_pc {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::from_rgba_unmultiplied(
                                                    255, 255, 0, 100,
                                                ),
                                            );

                                            if pc_moved {
                                                ui.scroll_to_rect(rect, Some(egui::Align::Center));
                                            }
                                        }
                                        ui.painter().galley(
                                            rect.min,
                                            galley,
                                            ui.visuals().text_color(),
                                        );
                                    }
                                });
                            });
                        });
                    });
                });
        }

        if self.show_fps {
            // Area allows us to place things freely on the screen.
            egui::Area::new(egui::Id::new("fps_counter"))
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_black_alpha(150))
                        .rounding(2.0)
                        .inner_margin(5.0)
                        .show(ui, |ui| {
                            // Get FPS from context.
                            let fps_text =
                                format!("FPS: {:.1}", ctx.input(|i| i.unstable_dt.recip()));
                            ui.label(
                                egui::RichText::new(fps_text)
                                    .color(egui::Color32::RED)
                                    .monospace(),
                            );
                        });
                });
        }
    }
}
