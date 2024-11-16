use crate::constants;
use std::collections::VecDeque;

/// # PPU
/// The PPU is the picture processing unit of our machine.
///
/// ## Video RAM
/// The Video RAM, or VRAM, are 8 KiB located in addresses 0x8000 to 0x9FFF.
/// A **memory bank** contains 384 tiles, or 3 tile blocks, so 6 KiB of tile data.
/// After that, it  has two maps of 1024 bytes each (32 rows of 32 bytes each), the
/// Background Tile Map. Each byte contains the tile number to be displayed.
/// The tiles are taken from the Tile Data Table, which is at either 0x8000-0x8FFF,
/// or 0x8800-ox97FF. In the first case, tiles are numbered as unsigned bytes (u8).
/// In the second case, the numbers are signed (i8), and tile 0 lies at 0x9000.
/// The Tile Data Table address can be selected via the LCDC register.
///
/// In total, a bank has 8 KiB of memory.
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
    start_dot: u32,
    /// Current dot within a frame, in [0,4560).
    fdot: u32,
    /// Current dot within the line, in [0,456).
    ldot: u32,

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
    /// Update LY.
    pub ly_update: bool,

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
    /// STAT2: LYC == LY flag.
    stat2: bool,
    /// STAT10: PPU mode.
    stat01: u8,

    /// SCY: Scroll Y position. Top coordinate of the visible 160x144 area within the BG map.
    scy: u8,
    /// SCX: Scroll X position. Left coordinate of the visible 160x144 area within the BG map.
    scx: u8,

    /// WY: Window Y position.
    wy: u8,
    /// WX: Window X position plus 7.
    wx: u8,
    /// Trigger for WY.
    wy_trigger: bool,
    /// Position WY.
    wy_pos: i32,
    /// BGP: Background palette register.
    bgp: u8,
    /// OBP0: Object palette 0.
    obp0: u8,
    /// OBP1: Object palette 1.
    obp1: u8,

    /// LCD interrupt mask for registers IE and IF.
    pub i_mask: u8,

    /// Whether we are in H-Blank region.
    pub hblank: bool,
    /// Flag that goes up when the screen is updated.
    pub updated: bool,
    pub data_available: bool,

    /// The palette.
    palette: [u8; 4 * 3],
    /// Screen buffer in RGBA8888.
    pub fb: [u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4],
    /// Color buffer.
    pub fb_col: [u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH],
}

impl PPU {
    pub fn new(start_dot: u32) -> Self {
        // The 4 GB shades in RGB888.
        let palette = [224, 248, 208, 136, 192, 112, 52, 104, 86, 8, 24, 32];
        // let palette = [24, 48, 08, 136, 192, 112, 52, 104, 86, 8, 24, 32];

        PPU {
            oam: [0xFF; constants::OAM_SIZE],
            vram: [0; constants::VRAM_SIZE],
            mode: 0,
            lcdc: 0,
            lcdc7: true,
            lcdc6: 0,
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
            ly_update: false,
            stat: 0,
            stat6: false,
            stat5: false,
            stat4: false,
            stat3: false,
            stat2: false,
            stat01: 0,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 7,
            wy_trigger: false,
            wy_pos: -1,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            i_mask: 0,
            hblank: false,
            updated: false,
            data_available: false,

            palette,
            fb: [0xff; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4],
            fb_col: [0x00; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH],
        }
    }

    pub fn reset(&mut self) {
        self.oam.fill(0xff);
        self.vram.fill(0x00);
        self.fb.fill(0xff);
        self.fb_col.fill(0x00);
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
        self.wy_trigger = false;
        self.wy_pos = -1;
        self.bgp = 0;
        self.obp0 = 0;
        self.obp1 = 1;
        self.i_mask = 0;
        self.hblank = false;
        self.updated = false;
        self.data_available = false;
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
            // STAT.
            0xFF41 => self.stat,
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

            // WX.
            0xFF4A => self.wx,
            // WY.
            0xFF4B => self.wy,

            _ => 0xFF,
        }
    }

    /// Write a byte to a PPU address.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                // VRAM only accessible when mode != 3
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
                self.stat = value;
                self.update_stat_flags();
            }
            // SCY.
            0xFF42 => self.scy = value,
            // SCX.
            0xFF43 => self.scx = value,
            // LY (read-only).
            0xFF44 => {}
            // LCY.
            0xFF45 => self.lyc = value,
            // DMA.
            0xFF46 => {
                // Writing to this register starts a DMA transfer from ROM/RAM to OAM.
                // The written value specifies the transfer source address divided by $100.
                // Source: $XX00-$XX9F (where XX is the written value).
                // Dest:   $FE00-FE9F
                // This is implemented in `memory.rs`.
            }
            // BGP.
            0xFF47 => self.bgp = value,
            // OBP0.
            0xFF48 => self.obp0 = value,
            // OBP1.
            0xFF49 => self.obp1 = value,

            // WX.
            0xFF4A => self.wx = value,
            // WY.
            0xFF4B => self.wy = value,
            _ => {}
        }
    }

    /// Performs a GPU cycle with the given number of t-cycles, or dots.
    /// 1 m-cycle has 4 dots, or t-cycles.
    /// Timing is divided between 154 lines, 144 during VDraw (modes 0, 2, 3),
    /// and 10 during VBlank. Each line takes 456 dots, and one frame takes
    /// 70224 dots.
    pub fn cycle(&mut self, t_cycles: u32) {
        if !self.is_ppu_enabled() {
            return;
        }
        self.hblank = false;

        let mut t_cycles_left = t_cycles;

        while t_cycles_left > 0 {
            let curr_cycles = if t_cycles_left >= 80 {
                80
            } else {
                t_cycles_left
            };
            self.fdot += curr_cycles;
            t_cycles_left -= curr_cycles;

            // Full line takes 114 ticks
            if self.fdot >= 456 {
                self.fdot -= 456;
                self.ly = (self.ly + 1) % 154;
                self.check_interrupt_lyc();

                // This is a VBlank line
                if self.ly >= 144 && self.mode != 1 {
                    self.update_mode(1);
                }
            }

            // This is a normal line
            if self.ly < 144 {
                if self.fdot <= 80 {
                    if self.mode != 2 {
                        self.update_mode(2);
                    }
                } else if self.fdot <= (80 + 172) {
                    // 252 cycles
                    if self.mode != 3 {
                        self.update_mode(3);
                    }
                } else {
                    // the remaining 204
                    if self.mode != 0 {
                        self.update_mode(0);
                    }
                }
            }
        }
    }

    /// Updates the PPU mode and triggers the necessary actions.
    /// Rendering happens when entering mode 0 (H-Blank).
    fn update_mode(&mut self, mode: u8) {
        self.mode = mode;

        if match self.mode {
            // H-blank.
            0 => {
                self.render_scanline();
                // Signal data available.
                self.data_available = true;
                self.hblank = true;
                self.stat3
            }

            // V-blank.
            1 => {
                self.wy_trigger = false;
                self.i_mask |= 0x01;
                self.updated = true;
                self.stat4
            }

            // OAM scan.
            2 => {
                self.data_available = false;
                self.stat5
            }

            // Draw.
            3 => {
                if self.lcdc5 && !self.wy_trigger && self.ly == self.wy {
                    self.wy_trigger = true;
                    self.wy_pos = -1;
                }
                // No data.
                false
            }
            _ => false,
        } {
            self.i_mask |= 0x02;
        }
    }

    /// Renders a single scanline.
    fn render_scanline(&mut self) {
        self.render_bgwin_scanline();
        self.render_sprites();
    }

    /// Fetches the 8 pixels of a row within a background or window tile.
    fn get_bgwin_tile_pixels(&mut self, tile_id: u8, line: u8, use_unsigned: bool) -> [u8; 8] {
        let tile_data_start = self.lcdc4;

        let tile_addr = if use_unsigned {
            tile_data_start + (tile_id as u16 * 16)
        } else {
            // Signed mode: convert tile_id to signed and offset within the -128 to 127 range
            let tile_id = (tile_id as i8 as i16 + 128) as u16;
            tile_data_start.wrapping_add(tile_id * 16)
        };

        let low_byte = self.read(tile_addr + line as u16 * 2);
        let high_byte = self.read(tile_addr + line as u16 * 2 + 1);
        let mut pixels = [0u8; 8];

        // Each bit pair in the bytes represents a pixel color
        for i in 0..8 {
            let color_id = ((high_byte >> (7 - i)) & 0x1) << 1 | ((low_byte >> (7 - i)) & 0x1);
            pixels[i] = color_id;
        }

        pixels
    }

    /// Fetches the 8 pixels of a row within a sprite tile.
    fn get_sprite_tile_pixels(
        &self,
        tile_id: u8,
        line: u8,
        attributes: u8,
        sprite_height: u8,
    ) -> [u8; 8] {
        let tile_addr = if sprite_height == 16 {
            // For 8x16 sprites, ignore lowest bit of tile_id.
            0x8000 + ((tile_id & 0xFE) as usize * 16) as u16
        } else {
            0x8000 + (tile_id as usize * 16) as u16
        };

        // Vertical flip.
        let line = if attributes & 0x40 != 0 {
            if sprite_height == 16 {
                15 - line
            } else {
                7 - line
            }
        } else {
            line
        };

        let low_byte = self.read(tile_addr + line as u16 * 2);
        let high_byte = self.read(tile_addr + line as u16 * 2 + 1);
        let mut pixels = [0u8; 8];

        // Horizontal flip.
        for i in 0..8 {
            let color_id = ((high_byte >> (7 - i)) & 0x1) << 1 | ((low_byte >> (7 - i)) & 0x1);
            let pixel_index = if attributes & 0x20 != 0 { 7 - i } else { i };
            pixels[pixel_index] = color_id;
        }

        pixels
    }

    /// Renders a single scanline of the background and window.
    fn render_bgwin_scanline(&mut self) {
        // Determine if the background and window are enabled.
        let bg_enabled = self.lcdc0;
        let win_enabled = self.lcdc5 && self.ly >= self.wy;

        // Determine if we’re using unsigned or signed tile IDs
        let use_unsigned = (self.lcdc & 0x10) != 0;

        for x in 0..constants::DISPLAY_WIDTH {
            // Calculate the tile coordinates and pixel position within the tile.
            let (tile_map, comp_x, comp_y, tile_x, tile_y) =
                if win_enabled && (x as u8) >= self.wx.wrapping_sub(7) {
                    let win_y = self.ly.wrapping_add(self.wy);
                    let tile_y = win_y / 8;
                    let win_x = x - self.wx.wrapping_sub(7) as usize;
                    let tile_x = win_x / 8;
                    (self.lcdc6, win_x, win_y, tile_x, tile_y)
                } else if bg_enabled {
                    let bg_y = self.ly.wrapping_add(self.scy);
                    let tile_y = bg_y / 8;
                    let bg_x = x.wrapping_add(self.scx as usize);
                    let tile_x = bg_x / 8;
                    (self.lcdc3, bg_x, bg_y, tile_x, tile_y)
                } else {
                    // Disable.
                    self.color(x, self.ly, 0);
                    continue;
                };

            // Calculate tile map address and fetch tile ID.
            let tile_map_addr = tile_map + (tile_y as u16 * 32) + tile_x as u16;
            let tile_id = self.read(tile_map_addr);

            // Get the specific row of pixels from the tile.
            let tile_line = (comp_y % 8) as u8;
            let tile_pixels = self.get_bgwin_tile_pixels(tile_id, tile_line, use_unsigned);

            // Get the background pixel and apply palette color.
            let color_idx = tile_pixels[comp_x % 8];
            let color = (self.bgp >> (color_idx * 2)) & 0x03;

            // Store the pixel color in the framebuffer.
            self.color(x, self.ly, color);
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

            // Check if the sprite overlaps with the current scanline
            if self.ly + 16 >= sprite_y && self.ly + 16 < sprite_y + sprite_height {
                sprites.push(Sprite {
                    y: sprite_y,
                    x: sprite_x,
                    tile_id,
                    attributes,
                });
                if sprites.len() >= constants::MAX_SPRITES_PER_LINE {
                    break; // Limit to 10 sprites per line as per Game Boy hardware
                }
            }
        }

        sprites
    }

    /// Renders a single scanline of sprites.
    fn render_sprites(&mut self) {
        if !self.lcdc1 {
            return;
        }
        let sprite_size = self.lcdc2;
        let sprites = self.get_sprites_on_scanline(sprite_size);

        for sprite in sprites.iter().rev() {
            let tile_line = (self.ly + 16 - sprite.y) % sprite_size;
            let pixels = self.get_sprite_tile_pixels(
                sprite.tile_id,
                tile_line,
                sprite.attributes,
                sprite_size,
            );

            for i in 0..8 {
                let x_pos = sprite.x.wrapping_sub(8) as usize + i;
                if x_pos >= constants::DISPLAY_WIDTH {
                    continue; // Ignore pixels outside screen bounds
                }

                let color_idx = pixels[i];
                if color_idx == 0 {
                    continue; // Skip transparent pixels
                }

                let palette = if sprite.attributes & 0x10 != 0 {
                    self.obp1
                } else {
                    self.obp0
                };
                let color = (palette >> (color_idx * 2)) & 0x03;

                // Priority handling: if bg/sprite priority bit is 0, sprite has priority over background
                let has_priority = sprite.attributes & 0x80 == 0;
                if has_priority
                    || self.fb_col[self.ly as usize * constants::DISPLAY_WIDTH + x_pos] == 0
                {
                    // Store the pixel color in the framebuffer
                    self.color(x_pos, self.ly, color);
                }
            }
        }
    }

    /// Sets the pixel at the given position to the given color id.
    fn color(&mut self, x: usize, y: u8, color_id: u8) {
        // Color frame buffer.
        self.fb_col[y as usize * constants::DISPLAY_WIDTH + x] = color_id;
        // R.
        self.fb[(y as usize * constants::DISPLAY_WIDTH + x) * 4 + 0] =
            self.palette[color_id as usize * 3];
        // G.
        self.fb[(y as usize * constants::DISPLAY_WIDTH + x) * 4 + 1] =
            self.palette[color_id as usize * 3 + 1];
        // B.
        self.fb[(y as usize * constants::DISPLAY_WIDTH + x) * 4 + 2] =
            self.palette[color_id as usize * 3 + 2];
        // A.
        self.fb[(y as usize * constants::DISPLAY_WIDTH + x) * 4 + 3] = 0xff;
    }

    fn clear_screen(&mut self) {
        for v in self.fb.iter_mut() {
            *v = 255;
        }
        for v in self.fb_col.iter_mut() {
            *v = 0;
        }
        self.updated = true;
    }

    /// Update STAT bit 2 (LYC==LY).
    fn check_interrupt_lyc(&mut self) {
        if self.stat6 && self.ly == self.lyc {
            self.i_mask |= 0b0000_0010;
        }
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

        if prev_lcd_status && !self.lcdc7 {
            // Screen went off.
            self.fdot = 0;
            self.ly = 0;
            self.mode = 0;
            self.wy_trigger = false;
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
        // LYC == LY flag (read only).
        //self.stat2 = self.stat & 0b0000_0100 == 0;
        // PPU Mode (read only).
        //self.stat01 = self.stat & 0b0000_0011;
    }

    /// Are the LCD and the PPU enabled?
    pub fn is_ppu_enabled(&self) -> bool {
        self.lcdc7
    }
}

#[derive(Debug, Clone, Copy)]
struct Sprite {
    y: u8,
    x: u8,
    tile_id: u8,
    attributes: u8,
}
