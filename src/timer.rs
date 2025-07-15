/// The timer registers and logic.
pub struct Timer {
    /// Divider. It is incremented internally every T-cycle, but only the upper 8 bits
    /// are readable.
    div_counter: u16,
    /// Detecting 1->0 edges.
    last_div_bit: bool,
    /// Timer counter. The most common software interface to the timers. It can be configured
    /// using TAC to increment at different rates.
    tima: u8,
    /// Modulo. When TIMA overflows, it is reset to the value in here.
    tma: u8,
    /// Enabled.
    enabled: bool,
    /// Step.
    step: u32,
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
            div_counter: 0,
            last_div_bit: false,
            tima: 0,
            tma: 0,
            enabled: false,
            step: 1024,
            i_mask: 0,
        }
    }

    /// Resets the state of the timer.
    pub fn reset(&mut self) {
        self.div_counter = 0;
        self.last_div_bit = false;
        self.tima = 0;
        self.tma = 0;
        self.enabled = false;
        self.step = 1024;
        self.i_mask = 0;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // Only the upper 8 bits of DIV are mapped to memory.
            0xFF04 => (self.div_counter >> 8) as u8,
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
                self.div_counter = 0;
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
        // DIV increments every M-cycle (4 T-cycles)
        for _ in 0..(t_cycles / 4) {
            self.div_counter = self.div_counter.wrapping_add(1);

            // Update TIMA on falling edge of selected bit
            if self.enabled {
                let bit = match self.step {
                    T256 => 9, // 4096 Hz
                    T4 => 3,   // 262144 Hz
                    T16 => 5,  // 65536 Hz
                    T64 => 7,  // 16384 Hz
                    _ => 9,
                };
                let new_bit = (self.div_counter >> bit) & 1 != 0;
                if self.last_div_bit && !new_bit {
                    // falling edge
                    self.tima = self.tima.wrapping_add(1);
                    if self.tima == 0 {
                        self.tima = self.tma;
                        self.i_mask |= 0b0000_0100;
                    }
                }
                self.last_div_bit = new_bit;
            }
        }
    }

    pub fn div(&self) -> u8 {
        (self.div_counter >> 8) as u8
    }
    pub fn div16(&self) -> u16 {
        self.div_counter
    }
}
