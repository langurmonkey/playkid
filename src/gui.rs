use crate::constants;
use crate::instruction::RunInstr;
use crate::machine::Machine;
use crate::uistate::UIState;
use egui::{
    CollapsingHeader, Color32, Context, FontFamily, FontId, Frame, RichText, ScrollArea, Sense,
    TextEdit, text::LayoutJob, vec2,
};
use egui_notify::{Anchor, Toasts};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub const BLUE: Color32 = Color32::from_rgb(66, 133, 244);
pub const GRAY: Color32 = Color32::from_rgb(127, 127, 127);
pub const DARKGRAY: Color32 = Color32::from_rgb(10, 10, 10);
pub const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
pub const CYAN: Color32 = Color32::from_rgb(0, 188, 212);
pub const MAGENTA: Color32 = Color32::from_rgb(233, 30, 99);
pub const GREEN: Color32 = Color32::from_rgb(15, 157, 88);
pub const RED: Color32 = Color32::from_rgb(219, 68, 55);
pub const YELLOW: Color32 = Color32::from_rgb(60, 52, 0);
pub const ORANGE: Color32 = Color32::from_rgb(255, 132, 0);

/// # GUI
/// The main GUI of Play Kid. Contains a menu bar, the 'About' window,
/// the FPS counter, and the debug panel.
pub struct Gui {
    /// Show about window.
    show_about: bool,
    /// Show debugger.
    pub show_debugger: bool,
    /// Show FPS.
    show_fps: bool,
    /// The FPS timer.
    fps_timer: f32,
    /// The current FPS value.
    current_fps: f32,
    /// Frame count.
    frame_count: f32,
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
    /// Toasts (notifications).
    toasts: Toasts,
    /// MPSC sender channel for ROM paths.
    pub load_tx: Sender<Option<PathBuf>>,
}

impl Gui {
    /// Create a `Gui`.
    pub fn new(show_debugger: bool, show_fps: bool, load_tx: Sender<Option<PathBuf>>) -> Self {
        Self {
            show_about: false,
            show_debugger,
            show_fps,
            fps_timer: 0.0,
            current_fps: 100.0,
            frame_count: 0.0,
            menu_timer: 0.0,
            last_mouse_pos: None,
            ui_state: UIState::new(),
            breakpoint_input: String::new(),
            breakpoint_error: false,
            logo_texture: None,
            last_pc: 0,
            toasts: Toasts::default().with_anchor(Anchor::BottomLeft),
            load_tx,
        }
    }

    pub fn clear_toasts(&mut self) {
        self.toasts.dismiss_all_toasts();
    }

    /// Adds an information toast with the given text.
    pub fn add_info_toast(&mut self, text: &str) {
        self.toasts
            .info(text)
            .duration(Some(Duration::from_secs(3)))
            .level(egui_notify::ToastLevel::Info)
            .closable(true);
    }

    /// Toggle state of FPS.
    pub fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
    }

    pub fn show_debugger(&mut self, show: bool) {
        self.show_debugger = show;
    }

    /// Create the UI using egui.
    pub fn ui(&mut self, ctx: &Context, machine: &mut Option<Machine>) {
        let mouse_pos = ctx.input(|i| i.pointer.hover_pos());
        let dt = ctx.input(|i| i.stable_dt);

        // Check for movement.
        let mouse_moved = if let (Some(current), Some(last)) = (mouse_pos, self.last_mouse_pos) {
            current != last
        } else {
            false
        };
        // Check sensors.
        let mouse_at_top = mouse_pos.is_some_and(|pos| pos.y < 30.0);
        let menu_in_use = ctx.input(|i| i.pointer.has_pointer())
            && ctx
                .layer_id_at(mouse_pos.unwrap_or_default())
                .is_some_and(|l| {
                    l.order == egui::Order::Foreground || l.order == egui::Order::Tooltip
                });

        // Toasts.
        self.toasts.show(ctx);

        // Update Timer (5s).
        if mouse_moved || mouse_at_top || menu_in_use {
            self.menu_timer = 5.0;
        } else {
            self.menu_timer -= dt;
        }
        self.last_mouse_pos = mouse_pos;

        // Top menu bar.
        if self.menu_timer > 0.0 {
            egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    let can_open = !self.ui_state.is_picking_file;
                    // File menu.
                    ui.menu_button("File", |ui| {
                        ui.add_enabled_ui(can_open, |ui| {
                            if ui.button("Open ROM...").clicked() {
                                self.ui_state.is_picking_file = true;
                                let tx = self.load_tx.clone();
                                let ctx = ctx.clone(); // To request a repaint when done
                                std::thread::spawn(move || {
                                    let task = rfd::AsyncFileDialog::new()
                                        .add_filter("Game Boy", &["gb", "gbc", "bin"])
                                        .pick_file();

                                    let result = pollster::block_on(task);
                                    let path_option =
                                        result.map(|handle| handle.path().to_path_buf());

                                    let _ = tx.send(path_option);
                                    ctx.request_repaint();
                                });
                                ui.close();
                            }
                        });

                        ui.separator();

                        if ui.button("About...").clicked() {
                            self.show_about = true;
                            ui.close();
                        }
                        if ui.button("Quit").clicked() {
                            self.ui_state.exit_requested = true;
                            ui.close();
                        }
                    });

                    // Machine-dependent menus.
                    let is_loaded = machine.is_some();

                    ui.add_enabled_ui(is_loaded, |ui| {
                        ui.menu_button("Graphics", |ui| {
                            if let Some(m) = machine {
                                ui.menu_button("Palette", |ui| {
                                    let current_palette = m.memory.ppu.get_palette_index();
                                    for (i, name) in crate::ppu::PALETTE_NAMES.iter().enumerate() {
                                        let i = i as u8;
                                        if ui.radio(current_palette == i, *name).clicked() {
                                            m.memory.ppu.set_palette(i);
                                        }
                                    }
                                });
                            }

                            if ui.button("Save screenshot").clicked() {
                                self.ui_state.screenshot_requested = true;
                                ui.close();
                            }
                        });

                        ui.menu_button("Machine", |ui| {
                            if ui.button("Reset CPU").clicked() {
                                if let Some(m) = machine {
                                    m.reset();
                                }
                                ui.close();
                            }
                            if ui.button("Debug panel...").clicked() {
                                self.show_debugger = true;
                                ui.close();
                            }
                            ui.checkbox(&mut self.show_fps, "Show FPS");
                        });
                    });
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
        // Only attempt to draw if the toggle is ON and the machine is SOME
        if self.show_debugger {
            if let Some(m) = machine {
                self.draw_debugger_window(ctx, m);
            } else {
                // Auto close debug panel if machine is gone.
                self.show_debugger = false;
            }
        }

        self.frame_count += 1.0;
        if self.show_fps {
            // Update FPS logic so that it only updates every second.
            let dt = ctx.input(|i| i.stable_dt);
            self.fps_timer += dt;

            if self.fps_timer >= 1.0 {
                // Compute average FPS value.
                self.current_fps = self.frame_count / self.fps_timer;
                self.fps_timer = 0.0;
                self.frame_count = 0.0;
            }
            // Area allows us to place things freely on the screen.
            egui::Area::new(egui::Id::new("fps_counter"))
                .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 25.0))
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_black_alpha(150))
                        .corner_radius(2.0)
                        .inner_margin(5.0)
                        .show(ui, |ui| {
                            // Get FPS from context.
                            let fps_text = format!("FPS: {:.1}", self.current_fps);
                            ui.label(
                                egui::RichText::new(fps_text)
                                    .color(egui::Color32::RED)
                                    .monospace(),
                            );
                        });
                });
        }
    }

    fn draw_debugger_window(&mut self, ctx: &Context, machine: &mut Machine) {
        egui::SidePanel::right("🐛 Debug Panel")
            .default_width(300.0)
            .resizable(false)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 40, 253)),
            )
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

                    ui.add_space(8.0);

                    // Main horizontal pane.
                    ui.columns(2, |columns| {
                        // LEFT: Instruction, CPU, PPU, JOYP, BREAKP.
                        columns[0].vertical(|ui| {
                            ui.allocate_space(vec2(320.0, 0.0));
                            // Current instruction.
                            let run_instr =
                                RunInstr::new(opcode, &machine.memory, &machine.registers);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;

                                ui.label(
                                    RichText::new(format!("${:04x}:", pc))
                                        .color(GRAY)
                                        .monospace()
                                        .strong(),
                                )
                                .on_hover_text("Program Counter (PC)");
                                ui.label(
                                    RichText::new(format!(" {}", run_instr.instruction_str()))
                                        .color(ORANGE)
                                        .monospace()
                                        .strong(),
                                )
                                .on_hover_text("Current instruction");
                                ui.label(
                                    RichText::new(format!("  {}", run_instr.operand_str()))
                                        .color(BLUE)
                                        .monospace()
                                        .strong(),
                                )
                                .on_hover_text("Operand");
                            });

                            ui.add_space(8.0);

                            // CPU.
                            CollapsingHeader::new("💻 CPU")
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
                                                ui.label(
                                                    RichText::new("State:")
                                                        .color(GRAY)
                                                        .font(FontId::new(
                                                            12.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .strong(),
                                                );
                                                ui.label(
                                                    RichText::new(
                                                        (if machine.halted {
                                                            "HALTED"
                                                        } else {
                                                            "RUNNING"
                                                        })
                                                        .to_string(),
                                                    )
                                                    .color(if machine.halted { RED } else { GREEN })
                                                    .font(FontId::new(16.0, FontFamily::Monospace))
                                                    .strong(),
                                                );
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
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            RichText::new("AF ")
                                                                .color(WHITE)
                                                                .font(FontId::new(
                                                                    12.0,
                                                                    FontFamily::Monospace,
                                                                ))
                                                                .strong(),
                                                        );
                                                        ui.label(
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
                                                            .strong(),
                                                        );
                                                    });
                                                    // BC.
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            RichText::new("BC ")
                                                                .color(WHITE)
                                                                .font(FontId::new(
                                                                    12.0,
                                                                    FontFamily::Monospace,
                                                                ))
                                                                .strong(),
                                                        );
                                                        ui.label(
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
                                                            .strong(),
                                                        );
                                                    });
                                                    // DE.
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            RichText::new("DE ")
                                                                .color(WHITE)
                                                                .font(FontId::new(
                                                                    12.0,
                                                                    FontFamily::Monospace,
                                                                ))
                                                                .strong(),
                                                        );
                                                        ui.label(
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
                                                            .strong(),
                                                        );
                                                    });
                                                    // HL.
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            RichText::new("HL ")
                                                                .color(WHITE)
                                                                .font(FontId::new(
                                                                    12.0,
                                                                    FontFamily::Monospace,
                                                                ))
                                                                .strong(),
                                                        );
                                                        ui.label(
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
                                                            .strong(),
                                                        );
                                                    });
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

                                                ui.label(
                                                    RichText::new(format!(
                                                        "{} {} {} {}",
                                                        z, n, h, c
                                                    ))
                                                    .color(CYAN)
                                                    .font(FontId::new(12.0, FontFamily::Monospace))
                                                    .strong(),
                                                );

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
                                                ui.monospace(format!("{:#06x}", mem.timer.div16()));
                                                ui.end_row();
                                            });
                                    });
                                });

                            ui.add_space(8.0);

                            // PPU.
                            CollapsingHeader::new("📋 PPU")
                                .default_open(false)
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

                            ui.add_space(8.0);

                            // JOYPAD.
                            CollapsingHeader::new("🎮 Joypad")
                                .default_open(true)
                                .show(ui, |ui| {
                                    let mem = &machine.memory;
                                    ui.label(
                                        RichText::new(format!(
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
                                        .strong(),
                                    );
                                });

                            ui.add_space(8.0);

                            // BREAKPOINTS.
                            CollapsingHeader::new("○ Breakpoints")
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Breakpoint controls.
                                        ui.allocate_ui(
                                            egui::vec2(120.0, ui.available_height()),
                                            |ui| {
                                                ui.vertical(|ui| {
                                                    ui.label("Add (Hex):");

                                                    ui.horizontal(|ui| {
                                                        let br_input = TextEdit::singleline(
                                                            &mut self.breakpoint_input,
                                                        )
                                                        .hint_text("$0123")
                                                        .font(egui::TextStyle::Monospace)
                                                        .desired_width(60.0);

                                                        // Use a conditional visual for errors
                                                        if self.breakpoint_error {
                                                            ui.visuals_mut().override_text_color =
                                                                Some(RED);
                                                        }

                                                        if ui.add(br_input).changed() {
                                                            self.breakpoint_error = false;
                                                        }
                                                        ui.visuals_mut().override_text_color = None;

                                                        if ui.button("+").clicked() {
                                                            let text = &self
                                                                .breakpoint_input
                                                                .strip_prefix("$")
                                                                .unwrap_or(&self.breakpoint_input);
                                                            if let Ok(addr) =
                                                                u16::from_str_radix(text, 16)
                                                            {
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

                                                            if let Ok(addr) =
                                                                u16::from_str_radix(text, 16)
                                                            {
                                                                machine
                                                                    .debug
                                                                    .delete_breakpoint(addr);
                                                            }
                                                        }
                                                    });

                                                    ui.add_space(4.0);
                                                    if ui.button("Clear all").clicked() {
                                                        machine.debug.clear_breakpoints();
                                                    }
                                                });
                                            },
                                        );

                                        ui.separator();

                                        // Scrollable list of breakpoints.
                                        Frame::NONE.fill(DARKGRAY).inner_margin(4.0).show(
                                            ui,
                                            |ui| {
                                                ui.vertical(|ui| {
                                                    ui.label("Active:");
                                                    let breakpoints: Vec<u16> = machine
                                                        .debug
                                                        .get_breakpoints_vec()
                                                        .to_vec();
                                                    ScrollArea::vertical()
                                                        .id_salt("bp_list")
                                                        .max_height(100.0)
                                                        .min_scrolled_width(200.0)
                                                        .auto_shrink([false; 2])
                                                        .show(ui, |ui| {
                                                            if breakpoints.is_empty() {
                                                                ui.label(
                                                                    RichText::new("-empty-")
                                                                        .color(BLUE)
                                                                        .monospace(),
                                                                );
                                                            } else {
                                                                for bp in breakpoints {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(
                                                                            RichText::new(format!(
                                                                                "${:04x}",
                                                                                bp
                                                                            ))
                                                                            .color(RED)
                                                                            .monospace(),
                                                                        );
                                                                        if ui
                                                                            .small_button("❌")
                                                                            .on_hover_text("Remove")
                                                                            .clicked()
                                                                        {
                                                                            machine
                                                                                .debug
                                                                                .delete_breakpoint(
                                                                                    bp,
                                                                                );
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        });
                                                });
                                            },
                                        );
                                    });
                                });
                        });
                        // RIGHT: Disassembly.
                        columns[1].vertical(|ui| {
                            ui.allocate_space(vec2(290.0, 0.0));
                            ui.heading("Code disassembly");

                            ui.add_space(8.0);

                            // Get the EXACT spacing egui uses between labels.
                            let spacing_y = ui.spacing().item_spacing.y;
                            let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
                            // The "True" height of one line entry.
                            let line_height = row_height + spacing_y;

                            let current_pc = machine.registers.pc;
                            let pc_moved = current_pc != self.last_pc;

                            // Use frame to make the background dark.
                            Frame::NONE
                                .fill(DARKGRAY)
                                .corner_radius(8.0)
                                .inner_margin(5.0)
                                .show(ui, |ui| {
                                    let mut scroll_area =
                                        ScrollArea::vertical().auto_shrink([false; 2]);

                                    if pc_moved {
                                        // We calculate the offset based on the spacing-adjusted height.
                                        let target_y = current_pc as f32 * line_height;

                                        let center_offset =
                                            target_y - (ui.available_height() / 2.0);

                                        scroll_area = scroll_area
                                            .vertical_scroll_offset(center_offset.max(0.0));
                                        self.last_pc = current_pc;
                                    }

                                    scroll_area.show_rows(
                                        ui,
                                        row_height,
                                        0xFFFF,
                                        |ui, row_range| {
                                            for addr in row_range {
                                                let is_current_pc = addr == current_pc as usize;

                                                let opcode = machine.memory.read8(addr as u16);
                                                let i = RunInstr::new(
                                                    opcode,
                                                    &machine.memory,
                                                    &machine.registers,
                                                );

                                                let mut instruction = LayoutJob::default();
                                                let font_id = FontId::monospace(12.0);

                                                let mut append_instr =
                                                    |text: String, color: Color32| {
                                                        instruction.append(
                                                            &text,
                                                            0.0,
                                                            egui::TextFormat {
                                                                font_id: font_id.clone(),
                                                                color,
                                                                ..Default::default()
                                                            },
                                                        );
                                                    };

                                                let addr_color =
                                                    if machine.debug.has_breakpoint(addr as u16) {
                                                        RED
                                                    } else {
                                                        GRAY
                                                    };
                                                append_instr(format!("${:04x} ", addr), addr_color);
                                                append_instr(
                                                    format!("{:<4} ", i.instruction_str()),
                                                    ORANGE,
                                                );
                                                append_instr(i.operand_str(), BLUE);

                                                let galley = ui.painter().layout_job(instruction);
                                                let (rect, response) = ui.allocate_exact_size(
                                                    vec2(ui.available_width(), row_height),
                                                    Sense::click(),
                                                );
                                                if response.clicked() {
                                                    machine.debug.toggle_breakpoint(addr as u16);
                                                }
                                                if is_current_pc {
                                                    ui.painter().rect_filled(rect, 0.0, YELLOW);

                                                    if pc_moved {
                                                        ui.scroll_to_rect(
                                                            rect,
                                                            Some(egui::Align::Center),
                                                        );
                                                    }
                                                }
                                                ui.painter().galley(
                                                    rect.min,
                                                    galley,
                                                    ui.visuals().text_color(),
                                                );
                                            }
                                        },
                                    );
                                });
                        });
                    });
                });
            });
    }
}
