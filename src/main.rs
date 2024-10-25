mod cartridge;
mod constants;
mod debug;
mod instruction;
mod machine;
mod memory;
mod registers;

use cartridge::Cartridge;
use clap::Parser;
use machine::GameBoy;
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
    /// Activate debug mode.
    #[arg(short, long)]
    debug: bool,
    /// Skip global checksum, header checksum, and logo sequence check.
    #[arg(short, long)]
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

    // Create a game boy with the given cartridge.
    let mut gameboy = GameBoy::new(&cart, args.debug);
    // Start the machine.
    gameboy.start();

    // Finish gracefully by returning ok.
    Ok(())
}
