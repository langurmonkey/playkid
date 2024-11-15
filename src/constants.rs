/// Work RAM size (8 KiB).
pub const WRAM_SIZE: usize = 8192;
/// Video RAM size (8 KiB).
pub const VRAM_SIZE: usize = 8 * 1024;
// High RAM size (128 bytes).
pub const HRAM_SIZE: usize = 128;
// OAM size (160 bytes).
pub const OAM_SIZE: usize = 4 * 40;
// IO size (128 bytes).
pub const IO_SIZE: usize = 128;
/// Display width.
pub const DISPLAY_WIDTH: usize = 160;
/// Display height.
pub const DISPLAY_HEIGHT: usize = 144;
/// CPU frequency [Hz].
pub const CPU_FREQ_HZ: usize = 4194304;
/// CPU period [ns].
pub const CPU_CLOCK_NS: u128 = (1000_000_000.0 / CPU_FREQ_HZ as f64) as u128;
/// Maximum number of sprites per line.
pub const MAX_SPRITES_PER_LINE: usize = 10;
