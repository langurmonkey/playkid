/// The timer registers and logic.
pub struct Timer {
    /// Divider. It is incremented internally every T-cycle, but only the upper 8 bits
    /// are readable.
    div: u8,
    /// Timer counter. The most common software interface to the timers. It can be configured
    /// using TAC to increment at different rates.
    tima: u8,
    /// Modulo. When TIMA overflows, it is reset to the value in here.
    tma: u8,
    /// Enabled.
    enabled: bool,
    /// Step.
    step: u32,
    /// Internal divider.
    i_div: u32,
    /// Internal counter.
    i_tima: u32,
    /// Timer interrupt mask for registers IE and IF.
    pub i_mask: u8,
}

const T4: u32 = 4 * 4;
const T16: u32 = 16 * 4;
const T64: u32 = 64 * 4;
const T256: u32 = 256 * 4;

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            enabled: false,
            step: 1024,
            i_div: 0,
            i_tima: 0,
            i_mask: 0,
        }
    }

    /// Resets the state of the timer.
    pub fn reset(&mut self) {
        self.div = 0;
        self.tima = 0;
        self.tma = 0;
        self.enabled = false;
        self.step = 1024;
        self.i_div = 0;
        self.i_tima = 0;
        self.i_mask = 0;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Only the upper 8 bits of DIV are mapped to memory.
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => {
                0xF8 | (if self.enabled { 0x4 } else { 0 })
                    | (match self.step {
                        T4 => 1,
                        T16 => 2,
                        T64 => 3,
                        T256 => 0,
                        _ => 0,
                    })
            }
            _ => panic!("Timer does not know address: {:#04X}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                self.i_div = 0;
                self.div = 0;
            }
            0xFF05 => {
                self.tima = value;
            }
            0xFF06 => {
                self.tma = value;
            }
            0xFF07 => {
                self.enabled = value & 0x4 != 0;
                self.step = match value & 0x3 {
                    1 => T4,
                    2 => T16,
                    3 => T64,
                    0 => T256,
                    _ => T256,
                };
            }
            _ => panic!("Timer does not know address: {:#04X}", address),
        };
    }

    /// Advances the timer(s) by the given amount of T-cycles.
    pub fn cycle(&mut self, t_cycles: u32) {
        self.i_div += t_cycles;
        while self.i_div >= 256 {
            self.div = self.div.wrapping_add(1);
            self.i_div = 0;
        }

        if self.enabled {
            self.i_tima += t_cycles;

            while self.i_tima >= self.step {
                self.tima = self.tima.wrapping_add(1);
                // Timer interrupt requested when TIMA overflows.
                if self.tima == 0 {
                    self.tima = self.tma;
                    self.i_mask |= 0b0000_0100;
                }
                self.i_tima -= self.step;
            }
        }
    }

    pub fn div(&self) -> u8 {
        self.div
    }
    pub fn div16(&self) -> u16 {
        self.i_div as u16
    }
}
