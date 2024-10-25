/// Addressable memory size using 16-bit addresses (0x0000 to 0xFFFF).
/// 64 KiB of addressable memory. This is address space
/// is mapped to internal, cartridge and other kinds of memory.
pub const MEM_SIZE: usize = 0xFFFF;
/// Work RAM size (8 KiB).
pub const WRAM_SIZE: usize = 8192;
/// Video RAM size (8 KiB).
pub const VRAM_SIZE: usize = 8192;
// High RAM size (128 bytes).
pub const HRAM_SIZE: usize = 128;
// OAM size (160 bytes).
pub const OAM_SIZE: usize = 4 * 40;
// IO size (128 bytes).
pub const IO_SIZE: usize = 128;
/// Bank size in bytes (8 KiB).
pub const BANK_SIZE: usize = 8192;
/// Number of tile blocks per bank.
pub const BANK_TILE_BLOCKS: usize = 3;
/// Number of maps in a bank.
pub const BANK_MAPS: usize = 2;
/// Tile size in bytes.
pub const TILE_SIZE: usize = 16;
/// Tile blocks contain 128 tiles each.
pub const TILE_BLOCK_SIZE: usize = 128 * TILE_SIZE;
/// Map size in bytes.
pub const MAP_SIZE: usize = 1024;
/// Display width.
pub const DISPLAY_WIDTH: usize = 160;
/// Display height.
pub const DISPLAY_HEIGHT: usize = 144;
