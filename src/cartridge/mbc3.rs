/// MBC1 Memory Bank Controller.
pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize, // Also used to select RTC registers.
    ram_enabled: bool,

    // Real Time Clock Registers.
    rtc_seconds: u8,
    rtc_minutes: u8,
    rtc_hours: u8,
    rtc_days_low: u8,
    rtc_days_high: u8,
    rtc_latch: u8,
}

impl MBC3 {
    pub fn new(rom_data: Vec<u8>, rom_size_code: u8, ram_size_code: u8) -> Self {
        let rom_size = match rom_size_code {
            0x00..=0x08 => 32768 << rom_size_code,
            _ => 32 * 1024,
        };

        let ram_size = match ram_size_code {
            0x01 => 2 * 1024,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        };

        let mut rom = vec![0; rom_size];
        let data_len = rom_data.len().min(rom_size);
        rom[..data_len].copy_from_slice(&rom_data[..data_len]);

        Self {
            rom,
            ram: vec![0; ram_size],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            rtc_seconds: 0,
            rtc_minutes: 0,
            rtc_hours: 0,
            rtc_days_low: 0,
            rtc_days_high: 0,
            rtc_latch: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let offset = (self.rom_bank * 0x4000) + (address as usize - 0x4000);
                self.rom[offset % self.rom.len()]
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // RAM/RTC Enable.
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,

            // ROM Bank Select (Full 7 bits, bank 0 becomes 1).
            0x2000..=0x3FFF => {
                let mut bank = (value & 0x7F) as usize;
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank = bank;
            }

            // RAM Bank / RTC Register Select.
            0x4000..=0x5FFF => {
                self.ram_bank = value as usize;
            }

            // Latch Clock Data.
            0x6000..=0x7FFF => {
                if self.rtc_latch == 0 && value == 1 {
                    // On real hardware, this would copy current time into
                    // the registers so the CPU reads a static value.
                    self.update_rtc();
                }
                self.rtc_latch = value;
            }
            _ => {}
        }
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        match self.ram_bank {
            0x00..=0x03 => {
                // Standard RAM Banks.
                let offset = (self.ram_bank * 0x2000) + (address as usize - 0xA000);
                self.ram[offset % self.ram.len()]
            }
            0x08 => self.rtc_seconds,
            0x09 => self.rtc_minutes,
            0x0A => self.rtc_hours,
            0x0B => self.rtc_days_low,
            0x0C => self.rtc_days_high,
            _ => 0xFF,
        }
    }

    pub fn write_ram(&mut self, address: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        match self.ram_bank {
            0x00..=0x03 => {
                let offset = (self.ram_bank * 0x2000) + (address as usize - 0xA000);
                let len = self.ram.len();
                if len > 0 {
                    self.ram[offset % len] = value;
                }
            }
            0x08 => self.rtc_seconds = value & 0x3F,
            0x09 => self.rtc_minutes = value & 0x3F,
            0x0A => self.rtc_hours = value & 0x1F,
            0x0B => self.rtc_days_low = value,
            0x0C => self.rtc_days_high = value,
            _ => {}
        }
    }

    fn update_rtc(&mut self) {}
}
