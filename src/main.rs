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
use machine::Machine;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "playkid")]
#[command(version = "1.0")]
#[command(about = "Not so fancy Game Boy emulator.", long_about = None)]
/// CLI arguments.
struct Args {
    /// Path to the input rom file to load.
    input: PathBuf,
    #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..12))]
    /// Display scale.
    scale: u8,
    /// Activate debug mode.
    #[arg(short, long)]
    debug: bool,
    /// Skip global checksum, header checksum, and logo sequence check.
    #[arg(long)]
    skipcheck: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let rom = args.input.as_path().to_str().unwrap();
    println!("Using rom file: {}", rom);

    if args.debug {
        println!("Debug mode is on");
    }

    // Load rom file into cartridge.
    let cart = Cartridge::new(rom, args.skipcheck).expect("Error reading rom file");

    let sdl_context = sdl2::init().unwrap();
    // Create a game boy with the given cartridge.
    let mut gameboy = Machine::new(&cart, &sdl_context, args.scale, args.debug);
    // Initialize the Game Boy state.
    gameboy.init();
    // Start the machine.
    gameboy.start();

    // Finish gracefully by returning ok.
    Ok(())
}
