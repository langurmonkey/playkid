/// The timer registers and logic.
pub struct Timer {
    /// Divider. It is incremented internally every T-cycle, but only the upper 8 bits
    /// are readable.
    divider: u16,
    /// Detecting 1->0 edges.
    last_div_bit: bool,
    /// Timer counter. The most common software interface to the timers. It can be configured
    /// using TAC to increment at different rates.
    tima: u8,
    /// Modulo. When TIMA overflows, it is reset to the value in here.
    tma: u8,
    /// Enabled.
    enabled: bool,
    /// Timer bit.
    timer_bit: u8,
    /// Timer interrupt mask for registers IE and IF.
    pub i_mask: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            last_div_bit: false,
            tima: 0,
            tma: 0,
            enabled: false,
            timer_bit: 9, // 4096 Hz -> bits 1-0 = 0b00
            i_mask: 0,
        }
    }

    /// Resets the state of the timer.
    pub fn reset(&mut self) {
        self.divider = 0;
        self.last_div_bit = false;
        self.tima = 0;
        self.tma = 0;
        self.enabled = false;
        self.timer_bit = 9;
        self.i_mask = 0;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Only the upper 8 bits of DIV are mapped to memory.
            0xFF04 => (self.divider >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => {
                let freq_bits = match self.timer_bit {
                    9 => 0b00, // 4096 Hz
                    3 => 0b01, // 262144 Hz
                    5 => 0b10, // 65536 Hz
                    7 => 0b11, // 16384 Hz
                    _ => 0b00, // default/fallback
                };
                0xF8 | (if self.enabled { 0x4 } else { 0 }) | freq_bits
            }
            _ => panic!("Timer does not know address: {:#04X}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                self.divider = 0;
                self.last_div_bit = ((self.divider >> self.timer_bit) & 1) != 0;
            }
            0xFF05 => {
                self.tima = value;
            }
            0xFF06 => {
                self.tma = value;
            }
            0xFF07 => {
                let old_bit = (self.divider >> self.timer_bit) & 1 != 0;

                // Update TAC values.
                let new_enabled = value & 0x4 != 0;
                let new_timer_bit = match value & 0x3 {
                    0b00 => 9, // 4096 Hz
                    0b01 => 3, // 262144 Hz
                    0b10 => 5, // 65536 Hz
                    0b11 => 7, // 16384 Hz
                    _ => 9,
                };
                let new_bit = (self.divider >> new_timer_bit) & 1 != 0;
                // Check for a falling edge across the change - even if timer is disabled.
                if old_bit && !new_bit {
                    self.increment_tima();
                }

                // Apply new config.
                self.enabled = new_enabled;
                self.timer_bit = new_timer_bit;
                self.last_div_bit = new_bit;
            }
            _ => panic!("Timer does not know address: {:#04X}", address),
        };
    }

    /// Advances the timer(s) by the given amount of T-cycles.
    pub fn cycle(&mut self, t_cycles: u32) {
        // DIV increments every M-cycle (4 T-cycles)
        for _ in 0..t_cycles {
            self.divider = self.divider.wrapping_add(1);
            let new_bit = (self.divider >> self.timer_bit) & 1 != 0;

            // Update TIMA on falling edge of selected bit
            if self.enabled {
                if self.last_div_bit && !new_bit {
                    self.increment_tima();
                }
            }
            self.last_div_bit = new_bit;
        }
    }

    fn increment_tima(&mut self) {
        // Falling edge
        self.tima = self.tima.wrapping_add(1);
        if self.tima == 0 {
            self.tima = self.tma;
            self.i_mask |= 0x04;
        }
    }

    pub fn div(&self) -> u8 {
        (self.divider >> 8) as u8
    }
    pub fn div16(&self) -> u16 {
        self.divider
    }
}
