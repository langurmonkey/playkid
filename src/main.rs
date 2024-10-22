mod cartridge;
mod constants;
mod instruction;
mod machine;
mod memory;
mod registers;

use cartridge::Cartridge;
use clap::Parser;
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
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let rom = args.input.as_path().to_str().unwrap();
    println!("Using rom file: {}", rom);

    // Load rom file into cartridge.
    let cart = Cartridge::new(rom, true).expect("Error reading rom file");
    Ok(())
}
