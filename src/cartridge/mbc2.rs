/// MBC2 Memory Bank Controller.
pub struct MBC2 {
    rom: Vec<u8>,
    ram: [u8; 512],
    rom_bank: u8,
    ram_enabled: bool,
}

impl MBC2 {
    pub fn new(rom_data: Vec<u8>, rom_size_code: u8) -> Self {
        let rom_size = match rom_size_code {
            0x00..=0x08 => 32768 << rom_size_code,
            _ => 32 * 1024,
        };

        let mut rom = vec![0; rom_size];
        let data_len = rom_data.len().min(rom_size);
        rom[..data_len].copy_from_slice(&rom_data[..data_len]);

        Self {
            rom,
            ram: [0; 512],
            rom_bank: 1,
            ram_enabled: false,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if (addr >> 8) & 0x01 == 0 {
                    self.ram_enabled = (val & 0x0F) == 0x0A;
                } else {
                    self.rom_bank = val & 0x0F;
                    if self.rom_bank == 0 {
                        self.rom_bank = 1;
                    }
                }
            }
            // Delegate to write_ram.
            0xA000..=0xBFFF => self.write_ram(addr, val),
            _ => {}
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => {
                let offset = (self.rom_bank as usize) * 0x4000;
                // Use modulo to safely wrap if the ROM is smaller than expected.
                self.rom[(offset + (addr as usize - 0x4000)) % self.rom.len()]
            }
            // Delegate to read_ram.
            0xA000..=0xBFFF => self.read_ram(addr),
            _ => 0xFF,
        }
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }
        // MBC2 RAM is only 512 bytes, it echoes/wraps until 0xBFFF.
        let index = (address as usize - 0xA000) % 512;
        // MBC2 only stores 4 bits. The upper 4 bits are usually 1s.
        self.ram[index] | 0xF0
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }
        let index = (address as usize - 0xA000) % 512;
        // Only the lower 4 bits are stored.
        self.ram[index] = value & 0x0F;
    }

    pub fn get_ram(&self) -> &[u8] {
        &self.ram
    }

    // Copy from the loaded buffer into the fixed array
    pub fn set_ram(&mut self, data: &[u8]) {
        let len = data.len().min(512);
        self.ram[..len].copy_from_slice(&data[..len]);
    }
}
