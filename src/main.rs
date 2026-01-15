mod apu;
mod canvas;
mod cartridge;
mod constants;
mod debug;
mod display;
mod instruction;
mod joypad;
mod machine;
mod memory;
mod ppu;
mod registers;
mod timer;

use cartridge::Cartridge;
use clap::Parser;
use colored::Colorize;
use machine::Machine;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "Play Kid",
    version = "1.0",
    about = "Minimalist Game Boy emulator for the cool kids.",
    author = "Toni Sagristà - tonisagrista.com",
    help_template = "{name} {version}\n{author}\n\n{about}\n\n{usage-heading} {usage}\n\n{all-args}"
)]
/// CLI arguments.
struct Args {
    /// Path to the input ROM file to load.
    input: PathBuf,
    #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..12))]
    /// Display scale.
    scale: u8,
    /// Activate debug mode.
    #[arg(short, long)]
    debug: bool,
    /// Print FPS every second to standard output.
    #[arg(short, long)]
    fps: bool,
    /// Skip global checksum, header checksum, and logo sequence check.
    #[arg(long)]
    skipcheck: bool,
}

fn main() -> io::Result<()> {
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
    cart.load_sram(rom);

    let sdl_context = sdl2::init().unwrap();

    // Create the machine.
    {
        // Create a game boy with the given cartridge.
        let mut gameboy = Machine::new(&mut cart, &sdl_context, args.scale, args.debug, args.fps);
        // Initialize the Game Boy state.
        gameboy.init();
        // Start the machine.
        gameboy.start();
    }

    // Save data back to disk after the machine stops running.
    cart.save_sram(rom);

    // Finish gracefully by returning OK.
    Ok(())
}
