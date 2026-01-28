use crate::constants;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = constants::NAME,
    version = env!("CARGO_PKG_VERSION"),
    about = "Minimalist Game Boy emulator for the cool kids.",
    author = "Toni Sagristà - tonisagrista.com",
    help_template = "{name} {version}\n{author}\n\n{about}\n\n{usage-heading} {usage}\n\n{all-args}"
)]
/// ## CLI Arguments
/// Contains the command line interface arguments of the desktop build
/// of Play Kid.
pub struct Args {
    /// Path to the input ROM file to load.
    pub input: Option<PathBuf>,
    /// Initial window scale. It can also be resized manually.
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..15))]
    pub scale: u8,
    /// Activate debug mode. Use `d` to stop program at any point.
    #[arg(short, long)]
    pub debug: bool,
    /// Show FPS counter. Use `f` to toggle on and off.
    #[arg(short, long)]
    pub fps: bool,
    /// Skip global checksum, header checksum, and logo sequence check.
    #[arg(long)]
    pub skipcheck: bool,
}

impl Args {
    /// Creates an Args instance with the default values.
    pub fn default() -> Args {
        Args {
            input: None,
            scale: 4,
            debug: false,
            fps: false,
            skipcheck: false,
        }
    }
}
