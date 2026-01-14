/// MBC1 Memory Bank Controller.
pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,

    // MBC1 state.
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    banking_mode: u8,

    // RAM size.
    ram_size: usize,
}

impl MBC1 {
    pub fn new(rom_data: Vec<u8>, rom_size_code: u8, ram_size_code: u8) -> Self {
        let rom_size = match rom_size_code {
            0 => 32 * 1024,   // 32KB, 2 banks
            1 => 64 * 1024,   // 64KB, 4 banks
            2 => 128 * 1024,  // 128KB, 8 banks
            3 => 256 * 1024,  // 256KB, 16 banks
            4 => 512 * 1024,  // 512KB, 32 banks
            5 => 1024 * 1024, // 1MB, 64 banks
            6 => 2048 * 1024, // 2MB, 128 banks
            7 => 4096 * 1024, // 4MB, 256 banks
            8 => 8192 * 1024, // 8MB, 512 banks
            _ => 32 * 1024,   // Default
        };

        let ram_size = match ram_size_code {
            0 => 0,          // No RAM
            1 => 2 * 1024,   // 2KB (unused value, but some ROMs use it)
            2 => 8 * 1024,   // 8KB
            3 => 32 * 1024,  // 32KB
            4 => 128 * 1024, // 128KB
            5 => 64 * 1024,  // 64KB
            _ => 0,
        };

        let mut rom = vec![0; rom_size];
        let rom_data_len = rom_data.len().min(rom_size);
        rom[..rom_data_len].copy_from_slice(&rom_data[..rom_data_len]);

        Self {
            rom,
            ram: vec![0xFF; ram_size],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            banking_mode: 0,
            ram_size,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Fixed ROM bank 0, always there.
            0x0000..=0x3FFF => self.rom[address as usize],
            // Switchable ROM bank.
            0x4000..=0x7FFF => {
                let bank = if self.banking_mode == 0 {
                    // In mode 0, upper bits are ignored for ROM.
                    self.rom_bank & 0x1F
                } else {
                    self.rom_bank & 0x7F
                };
                let offset = (bank * 0x4000) + (address as usize - 0x4000);
                if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // RAM enable.
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            // ROM bank number (lower 5 bits).
            0x2000..=0x3FFF => {
                let mut bank = value as usize & 0x1F;
                // Bank 0 is not allowed, becomes bank 1.
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank = (self.rom_bank & 0x60) | bank;
            }
            // RAM bank number or upper bits of ROM bank.
            0x4000..=0x5FFF => {
                let bits = value as usize & 0x03;
                if self.banking_mode == 0 {
                    // ROM banking mode: bits go to upper ROM bank.
                    self.rom_bank = (bits << 5) | (self.rom_bank & 0x1F);
                } else {
                    // RAM banking mode.
                    self.ram_bank = bits;
                }
            }
            // Banking mode select.
            0x6000..=0x7FFF => {
                self.banking_mode = value & 0x01;
            }
            _ => {}
        }
    }

    /// Read from currently mapped banks.
    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }

        let offset = if self.ram_size <= 8 * 1024 {
            // For 8KB RAM, ignore banking.
            address as usize - 0xA000
        } else {
            // For larger RAM, use banking.
            let bank = if self.banking_mode == 1 {
                self.ram_bank
            } else {
                0
            };
            (bank * 0x2000) + (address as usize - 0xA000)
        };

        if offset < self.ram.len() {
            self.ram[offset]
        } else {
            0xFF
        }
    }

    /// Write to RAM is unused in MBC1, but just in case...
    pub fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }

        let offset = if self.ram_size <= 8 * 1024 {
            address as usize - 0xA000
        } else {
            let bank = if self.banking_mode == 1 {
                self.ram_bank
            } else {
                0
            };
            (bank * 0x2000) + (address as usize - 0xA000)
        };

        if offset < self.ram.len() {
            self.ram[offset] = value;
        }
    }

    pub fn get_ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn set_ram(&mut self, data: Vec<u8>) {
        if data.len() == self.ram.len() {
            self.ram = data;
        }
    }
}
