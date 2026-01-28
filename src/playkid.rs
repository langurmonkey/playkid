#![deny(clippy::all)]

use crate::cartridge::Cartridge;
use crate::cli::Args;
use crate::constants;
use crate::eventhandler::EventHandler;
use crate::gui::Gui;
use crate::machine::Machine;

use colored::Colorize;
use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, TARGET_FRAME_DURATION};
use eframe::egui;
use gilrs::{Event, EventType, Gilrs};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

/// # Play Kid
/// The Play Kid application as an [eframe] app. Contains the
/// main state, the [Machine], the [Gui], and the LCD texture.
pub struct PlayKid {
    running: bool,
    gui: Gui,
    machine: Option<Machine>,
    last_update: Instant,
    // The GPU handle for the Game Boy screen.
    screen_texture: egui::TextureHandle,
    /// Game controller library.
    gilrs: Gilrs,
    /// CLI args.
    args: Args,
    /// MPSC receiver channel for ROM file paths.
    rx_load: Receiver<Option<PathBuf>>,
}

#[allow(dead_code)]
impl PlayKid {
    pub fn new(_cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        let (tx_load, rx_load) = channel();
        // Create LCD texture.
        let texture = _cc.egui_ctx.load_texture(
            "lcd_screen",
            egui::ColorImage::new(
                [DISPLAY_WIDTH, DISPLAY_HEIGHT],
                vec![egui::Color32::BLACK; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            ),
            egui::TextureOptions::NEAREST,
        );

        let mut machine = None;

        if let Some(path) = &args.input {
            if let Some(m) = Self::create_machine(path, args.skipcheck, args.debug) {
                machine = Some(m);
            }
        }

        let gui = Gui::new(args.debug, args.fps, tx_load);

        // Use Gilrs to handle gamepad input.
        let gilrs = Gilrs::new().unwrap();
        let gamepads = gilrs.gamepads();
        gamepads.for_each(move |(gid, g)| {
            println!(
                "{}: Gamepad {} detected: {} ",
                "OK".green(),
                gid,
                g.name().to_string().yellow()
            )
        });
        // Return instance.
        Self {
            running: true,
            gui,
            machine,
            last_update: Instant::now(),
            screen_texture: texture,
            gilrs,
            args,
            rx_load,
        }
    }
    pub fn new_wasm(_cc: &eframe::CreationContext<'_>, rom: String) -> Self {
        let (tx_load, rx_load) = channel();
        let texture = _cc.egui_ctx.load_texture(
            "gb_screen",
            egui::ColorImage::new(
                [DISPLAY_WIDTH, DISPLAY_HEIGHT],
                vec![egui::Color32::BLACK; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            ),
            egui::TextureOptions::NEAREST,
        );
        let args = Args::default();
        // Create machine.
        let machine =
            Self::create_machine(&std::path::PathBuf::from(&rom), args.skipcheck, args.debug);
        let gui = Gui::new(false, false, tx_load);
        Self {
            running: true,
            gui,
            machine,
            last_update: Instant::now(),
            screen_texture: texture,
            gilrs: Gilrs::new().unwrap(),
            args: Args::default(),
            rx_load,
        }
    }

    /// Helper to create a machine instance from a path
    fn create_machine(path: &std::path::PathBuf, skipcheck: bool, debug: bool) -> Option<Machine> {
        if let Some(rom_str) = path.to_str() {
            match Cartridge::new(rom_str, skipcheck) {
                Ok(mut cart) => {
                    cart.load_sram();
                    return Some(Machine::new(cart, debug));
                }
                Err(e) => eprintln!("Failed to load ROM: {}", e),
            }
        }
        None
    }

    /// Handle requests from the GUI.
    fn handle_ui_state(&mut self) {
        if self.gui.ui_state.exit_requested {
            self.running = false;
            self.gui.ui_state.exit_requested = false
        }
        if self.gui.ui_state.screenshot_requested {
            self.screenshot();
            self.gui.ui_state.screenshot_requested = false;
        }
    }

    /// Handle keyboard and mouse input.
    fn handle_inputs(&mut self, ctx: &egui::Context) {
        // If egui is focused on a text box or menu, don't pass keys to the GB
        if ctx.wants_keyboard_input() {
            return;
        }

        ctx.input(|i| {
            // GLOBAL INPUTS.
            // Exit.
            if i.key_pressed(egui::Key::Escape) {
                self.running = false;
            }
            // MACHINE INPUTS.
            if let Some(ref mut machine) = self.machine {
                let mut handled = false;

                // Debug.
                if !handled && i.key_pressed(egui::Key::D) {
                    let d = machine.debug.toggle_debugging();
                    machine.debug.set_paused(d);
                    self.gui.show_debugger(d);
                }

                // Reset.
                if i.key_pressed(egui::Key::R) {
                    machine.reset();
                    self.gui.add_info_toast("CPU reset");
                    handled = true;
                }

                // Palette change.
                if !handled && i.key_pressed(egui::Key::P) {
                    machine.memory.ppu.cycle_palette();
                    self.gui.add_info_toast(&format!(
                        "Palette changed to {}",
                        machine.memory.ppu.get_palette_name()
                    ));
                    handled = true;
                }

                // FPS.
                if !handled && i.key_pressed(egui::Key::F) {
                    self.gui.toggle_fps();
                    handled = true;
                }

                // Write SRAM.
                if !handled && i.key_pressed(egui::Key::W) {
                    if machine.memory.cart.is_dirty() {
                        machine.memory.cart.save_sram();
                        machine.memory.cart.consume_dirty();
                        self.gui.add_info_toast(&format!(
                            "SRAM file written: {:?}",
                            machine.memory.cart.get_sram_path()
                        ));
                    }
                    handled = true;
                }

                // Machine-specific input handling.
                if !handled {
                    handled = machine.handle_event(i);
                }

                // Screenshot logic
                if !handled && i.key_released(egui::Key::S) {
                    self.screenshot();
                }
            }
        });
    }

    /// Creates a screenshot from the front frame buffer of the PPU.
    fn screenshot(&mut self) {
        if let Some(ref machine) = self.machine {
            let fb = &machine.memory.ppu.fb_front;
            if let Ok(name) = save_screenshot(DISPLAY_WIDTH, DISPLAY_HEIGHT, fb) {
                println!("Screenshot saved: {}", name);
                self.gui
                    .add_info_toast(&format!("Screenshot saved: {}", name));
            }
        }
    }

    /// Handle controller/gamepad input.
    fn handle_controller_input(&mut self) {
        // Examine all events from the controller.
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            let mut handled = match event {
                EventType::Connected => {
                    let gamepad = self.gilrs.gamepad(id);
                    self.gui
                        .add_info_toast(&format!("Gamepad connected: {}", gamepad.name()));
                    true
                }
                EventType::Disconnected => {
                    self.gui.add_info_toast("Gamepad disconnected");
                    true
                }
                _ => false,
            };

            // Trigger Machine-specific controller logic.
            if !handled {
                if let Some(ref mut machine) = self.machine {
                    handled = match event {
                        EventType::ButtonReleased(button, _) => match button {
                            gilrs::Button::LeftTrigger => {
                                machine.memory.ppu.cycle_palette_rev();
                                true
                            }
                            gilrs::Button::RightTrigger => {
                                machine.memory.ppu.cycle_palette();
                                true
                            }
                            _ => false,
                        },
                        _ => false,
                    };

                    if !handled {
                        machine.memory.joypad.handle_controller_input(event);
                    }
                }
            }
        }
    }
}

impl eframe::App for PlayKid {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if ROM load has been requested via the channel.
        if let Ok(maybe_rom_path) = self.rx_load.try_recv() {
            self.gui.ui_state.is_picking_file = false;
            if let Some(rom_path) = maybe_rom_path {
                // A file was actually chosen.
                if let Some(ref mut m) = self.machine {
                    m.memory.cart.save_sram();
                }
                self.machine =
                    Self::create_machine(&rom_path, self.args.skipcheck, self.args.debug);
                self.last_update = std::time::Instant::now();
                self.gui
                    .add_info_toast(&format!("ROM Loaded: {:?}", rom_path));
            }
        }

        // Mouse/Kbd input.
        self.handle_inputs(ctx);
        // Controller input.
        self.handle_controller_input();
        // UI requests.
        self.handle_ui_state();

        // Check if we have stopped.
        if !self.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if let Some(ref mut machine) = self.machine {
            // Emulator Heartbeat.
            let mut frame_ready = false;
            let now = Instant::now();
            let mut dt = now.duration_since(self.last_update);

            // Cap dt to a few milliseconds.
            if dt > Duration::from_millis(100) {
                dt = TARGET_FRAME_DURATION;
                self.last_update = now - dt;
            }

            // Update.
            while dt >= TARGET_FRAME_DURATION {
                machine.update();
                dt -= TARGET_FRAME_DURATION;
                self.last_update += TARGET_FRAME_DURATION;
                frame_ready = true;
            }

            // Render LCD to texture.
            if frame_ready {
                let size = [DISPLAY_WIDTH, DISPLAY_HEIGHT];
                let color_image =
                    egui::ColorImage::from_rgba_unmultiplied(size, &machine.memory.ppu.fb_front);
                self.screen_texture
                    .set(color_image, egui::TextureOptions::NEAREST);
            }
        }

        // Render GUI.
        self.gui.ui(ctx, &mut self.machine);

        // Draw the LCD.
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                // Get the actual space left after egui::SidePanel/egui::TopBottomPanel take their share.
                let available_size = ui.available_size();

                // Calculate scale factors for both width and height.
                let scale_x = (available_size.x / DISPLAY_WIDTH as f32).floor();
                let scale_y = (available_size.y / DISPLAY_HEIGHT as f32).floor();

                // Use the smaller of the two to ensure it fits the "letterbox" or "pillarbox".
                let scale = scale_x.min(scale_y).max(1.0);

                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::new(&self.screen_texture).fit_to_exact_size(egui::vec2(
                            DISPLAY_WIDTH as f32 * scale,
                            DISPLAY_HEIGHT as f32 * scale,
                        )),
                    );
                });
            });

        // Force a repaint immediately to keep the emulator running.
        ctx.request_repaint();
    }

    /// Runs on exit.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(ref mut machine) = self.machine {
            machine.memory.cart.save_sram();
        }
    }
}

fn save_screenshot(
    width: usize,
    height: usize,
    frame: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};

    // Create an ImageBuffer from the raw pixels.
    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width as u32, height as u32, frame.to_vec())
            .ok_or("Failed to create image buffer from pixels")?;

    // Generate a filename with a timestamp.
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let filename = format!("screenshot_{}.png", timestamp);

    // Save as PNG.
    img.save(&filename)?;

    Ok(filename)
}
