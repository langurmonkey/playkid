#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod cartridge;
mod constants;
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

    if args.debug {
        println!("{}: Debug mode is on", "WARN".yellow());
    }

    // Load ROM file into cartridge.
    let mut cart = Cartridge::new(rom, args.skipcheck)
        .expect(&format!("{}: Error reading rom file", "ERR".red()));

    // Load existing save data from disk.
    cart.load_sram();

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels + egui")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };
    // Create Machine.
    let mut machine = Machine::new(&mut cart);

    // Event loop.
    let res = event_loop.run(|event, elwt| {
        // Handle input events.
        if input.update(&event) {
            // Close events.
            if input.key_pressed(KeyCode::Escape)
                || input.key_pressed(KeyCode::CapsLock)
                || input.close_requested()
            {
                elwt.exit();
                return;
            }

            // Update the scale factor.
            if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
            }

            machine.handle_event(&input);

            // Resize the window.
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
                framework.resize(size.width, size.height);
            }

            // Update Machine.
            machine.update();

            // Request a redraw.
            window.request_redraw();
        }

        match event {
            // Draw the current frame
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Render machine.
                let fb = machine.memory.ppu.fb;
                let frame = pixels.frame_mut();
                frame.copy_from_slice(&fb);

                // Prepare egui.
                framework.prepare(&window);

                // Render everything together.
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the world texture.
                    context.scaling_renderer.render(encoder, render_target);

                    // Render egui.
                    framework.render(encoder, render_target, context);

                    Ok(())
                });

                // Basic error handling.
                if let Err(err) = render_result {
                    log_error("pixels.render", err);
                    elwt.exit();
                }
            }
            Event::WindowEvent { event, .. } => {
                // Update egui inputs.
                framework.handle_event(&window, &event);
            }
            // Event::AboutToWait => {
            //     // This tells the OS we want to draw again as soon as the monitor is ready
            //     window.request_redraw();
            // }
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
