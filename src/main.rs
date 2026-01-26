#![deny(clippy::all)]

mod apu;
mod cartridge;
mod constants;
mod debugmanager;
mod eventhandler;
mod instruction;
mod joypad;
mod machine;
mod memory;
mod ppu;
mod registers;
mod timer;
mod uistate;

use crate::eventhandler::EventHandler;
use crate::gui::Framework;
use cartridge::Cartridge;
use clap::Parser;
use colored::Colorize;
use error_iter::ErrorIter as _;
use log::error;
use machine::Machine;
use pixels::{Error, Pixels, SurfaceTexture};
use std::path::PathBuf;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

mod gui;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

#[derive(Parser, Debug)]
#[command(
    name = "Play Kid",
    version = env!("CARGO_PKG_VERSION"),
    about = "Minimalist Game Boy emulator for the cool kids.",
    author = "Toni Sagristà - tonisagrista.com",
    help_template = "{name} {version}\n{author}\n\n{about}\n\n{usage-heading} {usage}\n\n{all-args}"
)]
/// CLI arguments.
struct Args {
    /// Path to the input ROM file to load.
    input: PathBuf,
    /// Initial window scale. It can also be resized manually.
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(4..15))]
    scale: u8,
    /// Activate debug mode. Use `d` to stop program at any point.
    #[arg(short, long)]
    debug: bool,
    /// Show FPS counter. Use `f` to toggle on and off.
    #[arg(short, long)]
    fps: bool,
    /// Skip global checksum, header checksum, and logo sequence check.
    #[arg(long)]
    skipcheck: bool,
}

/// Main entry point for Play Kid.
fn main() -> Result<(), Error> {
    let args = Args::parse();
    let rom = args.input.as_path().to_str().unwrap();
    println!("{}: Using rom file: {}", "OK".green(), rom);

    // Initialize window.
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let scale = args.scale as f64;
        let size = LogicalSize::new(
            constants::DISPLAY_WIDTH as f64 * scale,
            constants::DISPLAY_HEIGHT as f64 * scale,
        );
        WindowBuilder::new()
            .with_title("Play Kid")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // Initialize pixels renderer.
    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        // Create the pixels instance.
        let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        // Disable v-sync in case we want to run faster.
        pixels.enable_vsync(false);
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
            args.fps,
            args.debug,
        );

        (pixels, framework)
    };

    let mut last_update_inst = std::time::Instant::now();

    // Load ROM file into cartridge.
    let mut cart = Cartridge::new(rom, args.skipcheck)
        .expect(&format!("{}: Error reading rom file", "ERR".red()));

    // Load existing save data from disk.
    cart.load_sram();

    // Create Machine.
    let mut machine = Machine::new(&mut cart, args.debug);

    // Event loop.
    let res = event_loop.run(|event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
        // Handle input events.
        if input.update(&event) {
            // Check if egui wants the keyboard.
            let egui_wants_input = framework.egui_ctx.wants_keyboard_input();

            // Handle gamepad events.
            machine.memory.joypad.handle_controller_input();

            let mut handled = false;

            if input.key_released(KeyCode::Escape)
                || input.key_released(KeyCode::CapsLock)
                || input.close_requested()
            {
                // Close program.
                elwt.exit();
                return;
            }
            if !egui_wants_input {
                if input.key_released(KeyCode::KeyD) {
                    // Debug.
                    let d = machine.debug.toggle_debugging();
                    machine.debug.set_paused(d);
                    framework.gui.show_debugger(d);

                    // Resize window.
                    if d {
                        let _ = window.request_inner_size(winit::dpi::LogicalSize::new(1100, 800));
                    } else {
                        let _ = window.request_inner_size(winit::dpi::LogicalSize::new(500, 500));
                    }

                    handled = true;
                } else if input.key_released(KeyCode::KeyF) {
                    // FPS.
                    framework.gui.toggle_fps();
                    handled = true;
                } else if input.key_released(KeyCode::KeyP) {
                    // Cycle palette.
                    machine.memory.ppu.cycle_palette();
                    handled = true;
                } else if input.key_released(KeyCode::KeyR) {
                    // Reset.
                    machine.reset();
                    handled = true;
                } else if input.key_released(KeyCode::KeyW) {
                    // Write SRAM.
                    if machine.memory.cart.is_dirty() {
                        machine.memory.cart.save_sram();
                        machine.memory.cart.consume_dirty();
                    }
                    handled = true;
                } else if input.key_released(KeyCode::KeyS) {
                    // Capture the frame.
                    let frame = pixels.frame();
                    match save_screenshot(WIDTH, HEIGHT, frame) {
                        Err(err) => {
                            error!("{}: Failed to save screenshot: {}", "ERR".red(), err);
                        }
                        Ok(filename) => {
                            println!("{}: Screenshot saved: {}", "OK".green(), filename.blue());
                        }
                    }
                    handled = true;
                }

                // Handle events in machine.
                if !handled {
                    machine.handle_event(&input);
                }
            }

            // Handle GUI requests.
            if framework.gui.ui_state.exit_requested {
                // Consume.
                framework.gui.ui_state.exit_requested = false;
                // Quit.
                elwt.exit();
            }
            if framework.gui.ui_state.screenshot_requested {
                // Consume.
                framework.gui.ui_state.screenshot_requested = false;
                // Screenshot.
                let frame = pixels.frame();
                match save_screenshot(WIDTH, HEIGHT, frame) {
                    Err(err) => {
                        error!("{}: Failed to save screenshot: {}", "ERR".red(), err);
                    }
                    Ok(filename) => {
                        println!("{}: Screenshot saved: {}", "OK".green(), filename.blue());
                    }
                }
            }

            // Resize the window.
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
                framework.resize(size.width, size.height);
            }
        }

        match event {
            Event::AboutToWait => {
                // Control timing.
                if last_update_inst.elapsed() >= constants::TARGET_FRAME_DURATION {
                    // Update Machine.
                    machine.update();
                    last_update_inst += constants::TARGET_FRAME_DURATION;

                    // Signal that we have a fresh frame ready to show.
                    window.request_redraw();
                }
            }
            // Draw the current frame.
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Render machine if needed.
                let fb = machine.memory.ppu.fb_front;
                pixels.frame_mut().copy_from_slice(&fb);

                // Prepare egui.
                framework.prepare(&window, &mut machine);

                // Render pixels and egui.
                pixels
                    .render_with(|encoder, render_target, context| {
                        context.scaling_renderer.render(encoder, render_target);
                        framework.render(encoder, render_target, context);
                        Ok(())
                    })
                    .unwrap();
            }

            // Handle scaling and resizing
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        // Update egui's logical scaling.
                        framework.scale_factor(scale_factor);

                        // Sync the pixels surface to the new physical dimensions.
                        let new_size = window.inner_size();
                        if let Err(err) = pixels.resize_surface(new_size.width, new_size.height) {
                            log_error("pixels.resize_surface", err);
                            elwt.exit();
                        }
                        framework.resize(new_size.width, new_size.height);
                    }
                    // Handle standard window resizing.
                    WindowEvent::Resized(size) => {
                        if let Err(err) = pixels.resize_surface(size.width, size.height) {
                            log_error("pixels.resize_surface", err);
                            elwt.exit();
                        }
                        framework.resize(size.width, size.height);
                    }
                    _ => (),
                }

                // Update egui inputs for all other window events.
                framework.handle_event(&window, &event);
            }
            _ => (),
        }
    });

    // Save data back to disk after the machine stops running.
    cart.save_sram();

    // Result.
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{}: {method_name}() failed: {err}", "ERR".red());
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

fn save_screenshot(
    width: u32,
    height: u32,
    frame: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgba};

    // Create an ImageBuffer from the raw pixels.
    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, frame.to_vec())
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
