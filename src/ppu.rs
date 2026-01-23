use crate::constants;

use colored::Colorize;
use std::collections::HashMap;

/// Key for sprite tile row cache.
type SpriteTileKey = (u8, u8, bool, bool);

/// # PPU
/// The PPU is the picture processing unit of our machine.
///
/// ## Video RAM
/// The Video RAM, or VRAM, are 8 KiB located in addresses 0x8000 to 0x9FFF.
/// A **memory bank** contains 384 tiles, or 3 tile blocks, so 6 KiB of tile data.
/// After that, it has two maps of 1024 bytes each (32 rows of 32 bytes each), the
/// Background and Window tile maps. Each byte contains the tile number to be displayed.
/// The tiles are taken from the Tile Data Table, which is at either 0x8000-0x8FFF,
/// or 0x8800-0x97FF. In the first case, tiles are numbered as unsigned bytes (u8).
/// In the second case, the numbers are signed (i8), and tile 0 lies at 0x9000.
/// The Tile Data Table address can be selected via the LCDC register.
///
/// - A **tile** has 8x8 pixels, with a color depth of 2 bpp. Each tile is 16 bytes.
///   Tiles in a bank are typically grouped into blocks.
/// - A **tile block** contains 128 tiles of 16 bytes each, so 2048 bytes.
/// - A **map** contains 32x32=1024 bytes.
pub struct PPU {
    /// Object Attribute Memory.
    pub oam: [u8; constants::OAM_SIZE],
    /// Video RAM.
    pub vram: [u8; constants::VRAM_SIZE],

    /// There are four modes:
    /// - 0: HBlank
    /// - 1: VBlank
    /// - 2: OAM scan
    /// - 3: HDraw
    mode: u8,
    /// Start dot of the PPU.
    start_dot: u64,
    /// Current dot within a frame, in [0,4560).
    fdot: u64,
    /// Current dot within the line, in [0,456).
    ldot: u64,

    // The LCDC byte.
    pub lcdc: u8,
    /// LCD & PPU enable.
    lcdc7: bool,
    /// Window tile map.
    lcdc6: u16,
    /// Window enable.
    lcdc5: bool,
    /// BG & Window tile address.
    lcdc4: u16,
    /// BG tile map.
    lcdc3: u16,
    /// OBJ size.
    lcdc2: u8,
    /// OBJ enable.
    lcdc1: bool,
    /// BG & Window enable/priority.
    lcdc0: bool,

    /// LX: LCD X coordinate.
    pub lx: u8,
    /// LY: LCD Y coordinate.
    pub ly: u8,
    /// LYC: LY compare.
    pub lyc: u8,

    /// STAT: LCD status.
    pub stat: u8,
    /// STAT6: LYC int select.
    stat6: bool,
    /// STAT5: Mode2 int select.
    stat5: bool,
    /// STAT4: Mode1 int select.
    stat4: bool,
    /// STAT3: Mode0 int select.
    stat3: bool,

    /// SCY: Scroll Y position. Top coordinate of the visible 160x144 area within the BG map.
    scy: u8,
    /// SCX: Scroll X position. Left coordinate of the visible 160x144 area within the BG map.
    scx: u8,

    /// WY: Window Y position.
    wy: u8,
    /// WX: Window X position plus 7.
    wx: u8,
    /// Window line counter.
    wly: u16,
    /// WLY flag; internal window latch.
    wly_flag: bool,
    /// BGP: Background palette register.
    bgp: u8,
    /// OBP0: Object palette 0.
    obp0: u8,
    /// OBP1: Object palette 1.
    obp1: u8,

    /// LCD interrupt mask for registers IE and IF.
    pub i_mask: u8,

    /// Whether we are in HBlank region.
    pub hblank: bool,
    /// Did LY==LYC previously?
    last_ly_eq_lyc: bool,

    /// The palette.
    palette: [u8; 4 * 3],
    /// Index of the current palette.
    current_palette: u8,
    /// The buffer currently being drawn to by the PPU (Back Buffer)
    fb_back: [u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4],
    /// The buffer ready to be displayed (Front Buffer)
    pub fb_front: [u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4],
    /// Color ID buffer for priorities.
    pub priorities: [u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH],
}

/// Palette names.
pub const PALETTE_NAMES: [&str; 18] = [
    "Game Boy",
    "DMG Classic",
    "Pocket",
    "Web-Slinger",
    "Deep Blue",
    "Amber",
    "Fire Red",
    "Choco Mint",
    "GBC Yellow",
    "Rust Belt",
    "Cyberpunk",
    "Synthwave",
    "Ice Palace",
    "Neon Dream",
    "Bubblegum",
    "Toxic Waste",
    "Retro Future",
    "Ocean Sunset",
];
/// Palette colors.
pub const PALETTES: [[u8; 12]; 18] = [
    [224, 248, 208, 136, 192, 112, 52, 104, 86, 8, 24, 32],
    [155, 188, 15, 139, 172, 15, 48, 98, 48, 15, 56, 15],
    [200, 200, 168, 168, 168, 112, 104, 104, 88, 56, 48, 48],
    [240, 255, 255, 220, 20, 40, 30, 60, 160, 20, 10, 30],
    [199, 240, 216, 103, 182, 189, 11, 95, 164, 2, 5, 25],
    [251, 243, 209, 197, 182, 111, 122, 105, 49, 41, 32, 14],
    [255, 255, 255, 170, 50, 40, 85, 20, 10, 30, 5, 3],
    [187, 233, 191, 117, 167, 115, 74, 102, 70, 47, 46, 31],
    [255, 255, 140, 255, 140, 48, 168, 64, 24, 64, 32, 16],
    [242, 211, 171, 196, 114, 63, 115, 59, 46, 38, 20, 20],
    [255, 103, 231, 190, 38, 224, 100, 25, 150, 20, 10, 60],
    [110, 255, 255, 255, 0, 255, 100, 0, 150, 20, 0, 40],
    [230, 255, 255, 120, 190, 240, 50, 100, 180, 10, 20, 70],
    [180, 255, 255, 255, 0, 255, 90, 0, 180, 20, 0, 60],
    [255, 250, 220, 255, 160, 130, 230, 50, 110, 80, 10, 50],
    [230, 255, 0, 50, 220, 50, 10, 80, 90, 5, 20, 20],
    [100, 255, 255, 180, 140, 255, 255, 100, 50, 20, 10, 30],
    [255, 240, 150, 255, 120, 30, 50, 60, 180, 10, 20, 60],
];

impl PPU {
    pub fn new(start_dot: u64) -> Self {
        // Default palette.
        let palette = PALETTES[0];

        PPU {
            oam: [0xFF; constants::OAM_SIZE],
            vram: [0; constants::VRAM_SIZE],
            mode: 0,
            lcdc: 0,
            lcdc7: true,
            lcdc6: 0x9c00,
            lcdc5: true,
            lcdc4: 0,
            lcdc3: 0,
            lcdc2: 0,
            lcdc1: true,
            lcdc0: true,
            start_dot,
            fdot: start_dot,
            ldot: start_dot % 456,
            lx: 0,
            ly: 0,
            lyc: 0,
            stat: 0,
            stat6: false,
            stat5: false,
            stat4: false,
            stat3: false,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 7,
            wly: 0,
            wly_flag: false,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            i_mask: 0,
            hblank: false,
            last_ly_eq_lyc: false,

            palette,
            current_palette: 0,
            fb_front: [0xff; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4],
            fb_back: [0xff; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4],
            priorities: [0x01; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH],
        }
    }

    pub fn reset(&mut self) {
        self.oam.fill(0xff);
        self.vram.fill(0x00);
        self.fb_front.fill(0xff);
        self.fb_back.fill(0xff);
        self.priorities.fill(0x00);
        self.mode = 0;
        self.lcdc = 0;
        self.fdot = self.start_dot;
        self.ldot = self.start_dot % 456;
        self.ly = 0;
        self.lx = 0;
        self.lyc = 0;
        self.stat = 0;
        self.scx = 0;
        self.scy = 0;
        self.wx = 0;
        self.wy = 0;
        self.wly = 0;
        self.wly_flag = false;
        self.bgp = 0;
        self.obp0 = 0;
        self.obp1 = 1;
        self.i_mask = 0;
        self.hblank = false;
        self.last_ly_eq_lyc = false;
    }

    /// Read a byte from a PPU.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            // VRAM.
            0x8000..=0x9FFF => {
                if self.mode != 3 {
                    self.vram[(address - 0x8000) as usize]
                } else {
                    // During mode 3 VRAM is inaccessible.
                    0xFF
                }
            }
            // OAM.
            0xFE00..=0xFE9F => {
                if self.mode & 0x02 == 0 {
                    self.oam[(address - 0xfe00) as usize]
                } else {
                    // During modes 2 and 3 OAM is inaccessible.
                    0xFF
                }
            }
            // LCDC.
            0xFF40 => self.lcdc,
            // STAT - bit 7 is always 1 when STAT is read, hence the OR.
            0xFF41 => {
                if !self.is_ppu_enabled() {
                    // When LCD is off, bits 0-2 are 0. Bit 7 is always 1.
                    0x80
                } else {
                    self.stat | 0x80
                }
            }
            // SCY.
            0xFF42 => self.scy,
            // SCX.
            0xFF43 => self.scx,
            // LY.
            0xFF44 => self.ly,
            // LCY.
            0xFF45 => self.lyc,
            // DMA.
            0xFF46 => 0,
            // BGP.
            0xFF47 => self.bgp,
            // OBP0.
            0xFF48 => self.obp0,
            // OBP1.
            0xFF49 => self.obp1,

            // WY.
            0xFF4A => self.wy,
            // WX.
            0xFF4B => self.wx,

            _ => 0xFF,
        }
    }

    /// Write a byte to a PPU address.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                // VRAM only accessible when mode != 3.
                if self.mode != 3 {
                    self.vram[(address - 0x8000) as usize] = value;
                }
            }
            0xFE00..=0xFE9F => {
                // OAM inaccessible in modes 2 and 3.
                if self.mode & 0x02 == 0 {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }
            // LCDC.
            0xFF40 => {
                self.lcdc = value;
                self.update_lcdc_flags();
            }
            0xFF41 => {
                self.stat = (self.stat & 0x07) | (value & 0x78);
                self.update_stat_flags();
            }
            // SCY.
            0xFF42 => self.scy = value,
            // SCX.
            0xFF43 => self.scx = value,
            // LY (read-only).
            0xFF44 => {}
            // LCY.
            0xFF45 => {
                self.lyc = value;
                self.check_interrupt_lyc();
            }
            // DMA.
            0xFF46 => {
                // Writing to this register starts a DMA transfer from ROM/RAM to OAM.
                // The written value specifies the transfer source address divided by $100.
                // Source: $XX00-$XX9F (where XX is the written value).
                // Dest:   $FE00-FE9F
                // This is implemented in `memory.rs`.
            }
            // BGP.
            0xFF47 => {
                self.bgp = value;
            }
            // OBP0.
            0xFF48 => self.obp0 = value,
            // OBP1.
            0xFF49 => self.obp1 = value,

            // WY.
            0xFF4A => self.wy = value,
            // WX.
            0xFF4B => self.wx = value,
            _ => {}
        }
    }

    /// Performs a GPU cycle with the given number of t-cycles, or dots.
    /// 1 m-cycle has 4 dots, or t-cycles.
    /// Timing is divided between 154 lines, 144 during VDraw (modes 0, 2, 3),
    /// and 10 during VBlank. Each line takes 456 dots, and one frame takes
    /// 70224 dots.
    pub fn cycle(&mut self, t_cycles: u64) {
        if !self.is_ppu_enabled() {
            return;
        }
        self.hblank = false;

        self.fdot += t_cycles;

        // Determine the mode based on the CURRENT scanline (ly) and dots (fdot).
        let new_mode = if self.ly >= 144 {
            1 // VBlank.
        } else {
            match self.fdot {
                0..=80 => 2,   // OAM Scan.
                81..=252 => 3, // Drawing.
                _ => 0,        // HBlank.
            }
        };
        // Update mode if it changed.
        if new_mode != self.mode {
            self.update_mode(new_mode);
        }
        // Handle the transition to the NEXT scanline.
        if self.fdot >= 456 {
            self.fdot -= 456;
            self.ly = (self.ly + 1) % 154;

            self.check_interrupt_lyc();
        }
    }

    fn update_stat_ly_lyc(&mut self) {
        // Update mode bits in STAT (bits 0–1).
        self.stat = (self.stat & 0b1111_1100) | (self.mode & 0b0000_0011);

        // Update bit 2 (LYC == LY flag).
        if self.ly == self.lyc {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
    }

    /// Updates the PPU mode and triggers the necessary actions.
    /// Rendering happens when entering mode 0 (HBlank).
    fn update_mode(&mut self, mode: u8) {
        self.mode = mode;
        self.update_stat_ly_lyc();

        if match self.mode {
            // HBlank.
            0 => {
                self.render_scanline();
                self.hblank = true;
                self.stat3
            }

            // VBlank.
            1 => {
                // Frame is done, present it to front.
                self.fb_front.copy_from_slice(&self.fb_back);

                self.wly_flag = false;
                self.wly = 0;
                self.i_mask |= 0x01;
                self.stat4
            }

            // OAM scan.
            2 => {
                // Nothing else.
                self.stat5
            }

            // Draw.
            3 => {
                // Check if window should start on this line.
                if self.lcdc5 && self.ly == self.wy {
                    self.wly_flag = true;
                }
                // No data.
                false
            }
            _ => false,
        } {
            self.i_mask |= 0x02;
        }
    }

    /// Renders a single scan line.
    fn render_scanline(&mut self) {
        // Clear priorities for this scanline before rendering.
        let line_start = self.ly as usize * constants::DISPLAY_WIDTH;
        let line_end = line_start + constants::DISPLAY_WIDTH;
        self.priorities[line_start..line_end].fill(0);

        // Actual render of Background/Window and sprites.
        self.render_bgwin_scanline();
        self.render_sprites();
    }

    /// Fetches and combines the two bytes of pixel data (low/high) for a tile row,
    /// for the background/window.
    fn get_bgwin_tile_data(&mut self, tile_id: u8, line: u16, use_unsigned: bool) -> [u8; 8] {
        let tile_addr_base = if use_unsigned {
            // Unsigned mode: tile_id directly indexes from 0x8000.
            // 0x8000 + (tile_id * 16)
            self.lcdc4 + (tile_id as u16) * 16
        } else {
            // Signed mode: tile_id is treated as i8, tile 0 is at 0x9000.
            // 0x9000 + (tile_id as i8 * 16)
            // Which is equivalent to: 0x8800 + ((tile_id as i8 as i16 + 128) * 16)
            let signed_tile_id = tile_id as i8 as i16;
            // 0x8800 is the base, add (signed_id + 128) * 16 to get to 0x9000 for tile 0
            (self.lcdc4 as i16 + (signed_tile_id + 128) * 16) as u16
        };

        let tile_addr = tile_addr_base + (line * 2);

        let low_byte = self.read(tile_addr);
        let high_byte = self.read(tile_addr + 1);
        let mut pixels = [0u8; 8];

        // Each bit pair in the bytes represents a pixel color ID.
        for i in 0..8 {
            let color_id = ((high_byte >> (7 - i)) & 0x1) << 1 | ((low_byte >> (7 - i)) & 0x1);
            pixels[i] = color_id;
        }
        pixels
    }

    /// Renders the background and window for the current scan line.
    fn render_bgwin_scanline(&mut self) {
        let use_unsigned = (self.lcdc & 0x10) != 0;

        // Tracking variable for window rendering.
        let mut win_was_rendered = false;

        // Use 0xFF as sentinel since valid color IDs are 0-3.
        let mut bg_cache = [[0xffu8; 8]; 32];
        let mut win_cache = [[0xffu8; 8]; 32];

        for x in 0..constants::DISPLAY_WIDTH {
            let win_active_now = self.lcdc5 && self.wly_flag;
            // Window is active if WLY has been activated and WX is in range.
            let use_window = win_active_now && (x as u16) >= (self.wx.saturating_sub(7) as u16);

            let (tile_map_addr, px_x, px_y, tile_x, tile_y, cache) = if use_window {
                // Window pixel.
                win_was_rendered = true;
                // Window's internal X coordinate: current screen X minus window start position.
                // Window starts at (WX - 7), so internal window X is: x - (WX - 7).
                let win_x = x.wrapping_sub(self.wx.saturating_sub(7) as usize) as u16;
                let win_tile_x = (win_x / 8) & 31;
                let win_tile_y = (self.wly / 8) & 31;

                (
                    self.lcdc6,
                    (win_x & 0x07) as u8,
                    (self.wly) & 0x07,
                    win_tile_x,
                    win_tile_y,
                    &mut win_cache,
                )
            } else if self.lcdc0 {
                // Background pixel.
                let bg_x = self.scx as u32 + x as u32;
                let bg_y = self.scy.wrapping_add(self.ly);
                let bg_tile_x = (bg_x as u16 / 8) & 0x1f;
                let bg_tile_y = (bg_y as u16 / 8) & 0x1f;

                (
                    self.lcdc3,
                    bg_x as u8 & 0x07,
                    bg_y as u16 & 0x07,
                    bg_tile_x,
                    bg_tile_y,
                    &mut bg_cache,
                )
            } else {
                // LCDC0 disabled: render white background (color 0).
                // Still need to set priorities for sprite rendering!
                self.priorities[self.ly as usize * constants::DISPLAY_WIDTH + x] = 0;
                self.color(x, self.ly, 0);
                continue;
            };

            // Fetch the tile data if not already cached.
            let tile_index = tile_y * 32 + tile_x;

            if cache[tile_x as usize][0] == 0xff {
                let tile_id = self.read(tile_map_addr + tile_index);
                let tile_data = self.get_bgwin_tile_data(tile_id, px_y, use_unsigned);
                cache[tile_x as usize] = tile_data;
            }

            // Get the color index from the tile data.
            let tile_pixels = cache[tile_x as usize];
            let color_idx = tile_pixels[px_x as usize];
            let color = (self.bgp >> (color_idx * 2)) & 0x03;

            // Store the raw ID for sprite priority checks.
            self.priorities[self.ly as usize * constants::DISPLAY_WIDTH + x] = color_idx;
            // Render the pixel
            self.color(x, self.ly, color);
        }

        // Increment WLY only if the window was rendered in this scanline.
        if win_was_rendered {
            self.wly += 1;
        }
    }

    /// Fetches the sprite attributes from OAM.
    fn get_sprites_on_scanline(&self, sprite_height: u8) -> Vec<Sprite> {
        let mut sprites = Vec::new();

        for i in 0..40 {
            let base = i * 4;
            let sprite_y = self.oam[base];
            let sprite_x = self.oam[base + 1];
            let tile_id = self.oam[base + 2];
            let attributes = self.oam[base + 3];

            // Check if the sprite overlaps with the current scan line.
            if self.ly + 16 >= sprite_y && self.ly + 16 < sprite_y + sprite_height {
                sprites.push(Sprite {
                    y: sprite_y,
                    x: sprite_x,
                    tile_id,
                    attributes,
                    oam_index: i as u8,
                });

                // Limit to 10 sprites per line as per Game Boy hardware.
                if sprites.len() >= constants::MAX_SPRITES_PER_LINE {
                    break;
                }
            }
        }

        sprites
    }

    /// Fetches and combines the tow bytes of pixel data (low/high) of a row within a sprite tile.
    fn get_sprite_tile_pixels(
        &self,
        tile_id: u8,
        line: u8,
        attributes: u8,
        sprite_size_8x16: bool,
        cache: &mut HashMap<SpriteTileKey, [u8; 8]>,
    ) -> [u8; 8] {
        let vflip = attributes & 0x40 != 0;
        let hflip = attributes & 0x20 != 0;

        // For 8x16 sprites, line can be 0..15.
        let (tile_id, effective_line) = if sprite_size_8x16 {
            // Determine top or bottom tile based on line.
            // Note: tile_id's LSB is ignored for 8x16 sprites, so tile_id & 0xFE is top tile.
            let top_tile = tile_id & 0xFE;
            let (tile, line_in_tile) = if vflip {
                // Flip vertical line within the 16 lines.
                let flipped_line = 15 - line;
                if flipped_line < 8 {
                    (top_tile, flipped_line)
                } else {
                    (top_tile | 1, flipped_line - 8)
                }
            } else {
                if line < 8 {
                    (top_tile, line)
                } else {
                    (top_tile | 1, line - 8)
                }
            };
            (tile, line_in_tile)
        } else {
            // 8x8 sprites.
            let effective_line = if vflip { 7 - line } else { line };
            (tile_id, effective_line)
        };

        let key = (tile_id, effective_line, hflip, vflip);

        if let Some(cached) = cache.get(&key) {
            return *cached;
        }

        let tile_addr = 0x8000 + (tile_id as u16) * 16;
        let low_byte = self.read(tile_addr + (effective_line as u16) * 2);
        let high_byte = self.read(tile_addr + (effective_line as u16) * 2 + 1);

        let mut pixels = [0u8; 8];
        // Horizontal flip.
        for i in 0..8 {
            let color_id = ((high_byte >> (7 - i)) & 0x1) << 1 | ((low_byte >> (7 - i)) & 0x1);
            let pixel_index = if attributes & 0x20 != 0 { 7 - i } else { i };
            pixels[pixel_index] = color_id;
        }

        // Insert into cache.
        cache.insert(key, pixels);

        pixels
    }

    /// Renders a single scan line of sprites.
    fn render_sprites(&mut self) {
        if !self.lcdc1 {
            return;
        }
        let sprite_size = self.lcdc2;
        let sprites = self.get_sprites_on_scanline(sprite_size);
        let mut sprites = sprites;

        // Game Boy priority: X asc, then OAM index asc.
        sprites.sort_by(|a, b| {
            if a.x != b.x {
                a.x.cmp(&b.x)
            } else {
                a.oam_index.cmp(&b.oam_index)
            }
        });

        // Cache for decoded sprite rows.
        // - Key: (tile_id, line, hflip, vflip)
        // - Value: [u8; 8] for the  pixels
        let mut tile_row_cache = HashMap::new();

        for sprite in sprites.iter().rev() {
            self.render_sprite(sprite, &mut tile_row_cache);
        }
    }

    fn render_sprite(&mut self, sprite: &Sprite, cache: &mut HashMap<SpriteTileKey, [u8; 8]>) {
        let sprite_size = self.lcdc2;
        let tile_line = (self.ly + 16 - sprite.y) % sprite_size;
        let pixels = self.get_sprite_tile_pixels(
            sprite.tile_id,
            tile_line,
            sprite.attributes,
            sprite_size == 16,
            cache,
        );

        for i in 0..8 {
            let x_pos = (sprite.x as i16) - 8 + (i as i16);
            if x_pos < 0 || x_pos >= constants::DISPLAY_WIDTH as i16 {
                continue; // Ignore pixels outside screen bounds.
            }
            let x_pos = x_pos as usize;

            let color_idx = pixels[i];
            if color_idx == 0 {
                continue; // Skip transparent pixels.
            }

            // Fetch BG color ID from priorities cache.
            let bg_color_id = self.priorities[self.ly as usize * constants::DISPLAY_WIDTH + x_pos];

            // Priority logic:
            // Bit 7 (OBJ-to-BG Priority):
            //   0 = OBJ above BG (always draw sprite if not transparent)
            //   1 = OBJ behind BG colors 1-3 (only draw if BG is color 0)
            let obj_behind_bg = (sprite.attributes & 0x80) != 0;
            let sprite_has_priority = if obj_behind_bg {
                // Priority bit set: only draw sprite over BG color 0.
                bg_color_id == 0
            } else {
                // Priority bit clear: always draw sprite (over any BG color).
                true
            };

            if sprite_has_priority {
                let palette = if sprite.attributes & 0x10 != 0 {
                    self.obp1
                } else {
                    self.obp0
                };
                let color = (palette >> (color_idx * 2)) & 0x03;

                self.color(x_pos, self.ly, color);
            }
        }
    }

    /// Sets the pixel at the given position to the given color id.
    // In PPU struct, ensure priorities stores the raw Color ID (0-3)
    fn color(&mut self, x: usize, y: u8, paletted_color: u8) {
        let pos = y as usize * constants::DISPLAY_WIDTH + x;
        let base = paletted_color as usize * 3;

        // RGBA, in order.
        self.fb_back[pos * 4 + 0] = self.palette[base];
        self.fb_back[pos * 4 + 1] = self.palette[base + 1];
        self.fb_back[pos * 4 + 2] = self.palette[base + 2];
        self.fb_back[pos * 4 + 3] = 0xff;
    }

    fn clear_screen(&mut self) {
        // Get the first palette color (RGB888 format).
        let (r, g, b) = (self.palette[0], self.palette[1], self.palette[2]);

        self.fb_front.chunks_exact_mut(4).for_each(|chunk| {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = 0xff;
        });
        self.fb_back.chunks_exact_mut(4).for_each(|chunk| {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = 0xff;
        });

        // Set the priorities to all ones.
        self.priorities.fill(0xff);
    }

    /// Update STAT bit 2 (LYC==LY).
    fn check_interrupt_lyc(&mut self) {
        self.update_stat_ly_lyc();
        let coincidence = self.ly == self.lyc;

        // Update bit 2 of STAT.
        if coincidence {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }

        // Trigger interrupt only on rising edge.
        if self.stat6 && coincidence && !self.last_ly_eq_lyc {
            self.i_mask |= 0x02;
        }

        self.last_ly_eq_lyc = coincidence;
    }

    /// This method updates the LCDC flags from the current value
    /// in the byte `self.lcdc`.
    fn update_lcdc_flags(&mut self) {
        let prev_lcd_status = self.lcdc7;
        self.lcdc7 = self.lcdc & 0b1000_0000 != 0;
        self.lcdc6 = if self.lcdc & 0b0100_0000 == 0 {
            0x9800
        } else {
            0x9C00
        };
        self.lcdc5 = self.lcdc & 0b0010_0000 != 0;
        self.lcdc4 = if self.lcdc & 0b0001_0000 == 0 {
            // Signed access.
            0x8800
        } else {
            // Unsigned access.
            0x8000
        };
        self.lcdc3 = if self.lcdc & 0b0000_1000 == 0 {
            0x9800
        } else {
            0x9C00
        };
        self.lcdc2 = if self.lcdc & 0b0000_0100 == 0 { 8 } else { 16 };
        self.lcdc1 = self.lcdc & 0b0000_0010 != 0;
        self.lcdc0 = self.lcdc & 0b0000_0001 != 0;

        // Transition from LCD ON to OFF.
        if prev_lcd_status && !self.lcdc7 {
            self.fdot = 0;
            self.ly = 0;
            self.mode = 0;
            self.wly_flag = false;
            self.clear_screen();
        }
    }

    /// This method updates the STAT flags from the current value
    /// in the byte `self.stat`.
    fn update_stat_flags(&mut self) {
        // LYC int select (rw).
        self.stat6 = self.stat & 0b0100_0000 != 0;
        // Mode 2 int select (rw).
        self.stat5 = self.stat & 0b0010_0000 != 0;
        // Mode 1 int select (rw).
        self.stat4 = self.stat & 0b0001_0000 != 0;
        // Mode 0 int select (rw).
        self.stat3 = self.stat & 0b0000_1000 != 0;
    }

    /// Are the LCD and the PPU enabled?
    pub fn is_ppu_enabled(&self) -> bool {
        self.lcdc7
    }

    /// Set the palette for rendering.
    pub fn cycle_palette(&mut self) {
        self.current_palette = (self.current_palette + 1) % PALETTES.len() as u8;
        self.palette = PALETTES[self.current_palette as usize];
        println!(
            "{}: Palette changed to {}",
            "OK".green(),
            PALETTE_NAMES[self.current_palette as usize].yellow()
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct Sprite {
    y: u8,
    x: u8,
    tile_id: u8,
    attributes: u8,
    oam_index: u8,
}
