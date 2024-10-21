/// # Registers
/// We have 7 1-bit registers (`a`, `b`, `c`, `d`, `e`, `h`, `l`) which can be accessed individually,
/// or together as 16 bits, in the combinations `af`, `bc`, `de` and `hl`.
/// We also have the flags register, `f`, whose 4 most significant bits are the flags
/// zero `z`, subtraction `n`, half-carry `h` and carry `c`.
/// Additionally, we have two 16-bit special registers, the stack pointer `sp`, and
/// the program counter `pc`.
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    /// Flags register.
    /// - Bit 7: `z`, zero flag
    /// - Bit 6: `n`, subtraction flag (BCD)
    /// - Bit 5: `h`, half-carry flag (BCD)
    /// - Bit 4: `c`, carry flag
    pub f: u8,
    pub h: u8,
    pub l: u8,
    /// Stack pointer register.
    pub sp: u16,
    /// Program counter register.
    pub pc: u16,
}

impl Registers {
    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }
    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }
    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }
    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }
    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    pub fn get_hl_plus(&mut self) -> u16 {
        let result = self.get_hl();
        self.set_hl(result + 1);
        result
    }
    pub fn get_hl_minus(&mut self) -> u16 {
        let result = self.get_hl();
        self.set_hl(result - 1);
        result
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
    /// Set the zero flag `z` in the `f` register (pos 7).
    /// The least significant bit of `value` is used.
    pub fn set_flag_z(&mut self, value: u8) {
        self.f = (self.f & 0b0111_1111) | ((value & 0b0000_0001) << 7)
    }
    /// Set the zero flag `z`.
    pub fn z(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b0111_1111) | (v << 7)
    }
    /// Get the zero flag `z` from the `f` register (pos 7).
    pub fn get_flag_z(&self) -> bool {
        self.f & 0x80 > 0
    }
    /// Set the subtraction flag `n` in the `f` register (pos 6).
    /// The least significant bit of `value` is used.
    pub fn set_flag_n(&mut self, value: u8) {
        self.f = (self.f & 0b1011_1111) | ((value & 0b0000_0001) << 6)
    }
    /// Set the subtraction flag `n`.
    pub fn n(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b1011_1111) | (v << 6)
    }
    /// Get the subtraction flag `n` from the `f` register (pos 6).
    pub fn get_flag_n(&self) -> bool {
        self.f & 0x40 > 0
    }
    /// Set the half-carry flag `h` in the `f` register (pos 5).
    /// The least significant bit of `value` is used.
    pub fn set_flag_h(&mut self, value: u8) {
        self.f = (self.f & 0b1101_1111) | ((value & 0b0000_0001) << 5)
    }
    /// Set the half-carry flag `h`.
    pub fn h(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b1101_1111) | (v << 5)
    }
    /// Get the half-carry flag `h` from the `f` register (pos 5).
    pub fn get_flag_h(&self) -> bool {
        self.f & 0x20 > 0
    }
    /// Set the carry flag `c` in the `f` register (pos 4).
    /// The least significant bit of `value` is used.
    pub fn set_flag_c(&mut self, value: u8) {
        self.f = (self.f & 0b1110_1111) | ((value & 0b0000_0001) << 4)
    }
    /// Set the carry flag `c`.
    pub fn c(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b1110_1111) | (v << 4)
    }
    /// Get the carry flag `c` from the `f` register (pos 4).
    pub fn get_flag_c(&self) -> bool {
        self.f & 0x10 > 0
    }
}
