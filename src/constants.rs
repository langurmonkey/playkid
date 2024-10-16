/// Addressable memory size using 16-bit addresses (0x0000 to 0xFFFF).
/// 64 KiB of addressable memory. This is address space
/// is mapped to internal, cartridge and other kinds of memory.
pub const MEM_SIZE: usize = 0xFFFF;
/// Bank size in bytes.
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