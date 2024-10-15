/// We have 7 1-bit registers (`a`, `b`, `c`, `d`, `e`, `h`, `l`) which can be accessed individually,
/// or together as 16 bits, in the combinations `af`, `bc`, `de` and `hl`.
/// We also have the flags register, `f`, whose 4 most significant bits are the flags
/// zero `z`, subtraction `n`, half-carry `h` and carry `c`.
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    /// Flags register.
    /// - Bit 7: `z`, zero flag
    /// - Bit 6: `n`, subtraction flag (BCD)
    /// - Bit 5: `h`, half-carry flag (BCD)
    /// - Bit 4: `c`, carry flag
    f: u8,
    h: u8,
    l: u8,
    /// Stack pointer register.
    sp: u16,
    /// Program counter register.
    pc: u16,
}

impl Registers {
    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }
    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }
    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }
    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }
    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
    /// Set the zero flag `z` in the `f` register (pos 7).
    /// The least significant bit of `value` is used.
    fn set_flag_z(&mut self, value: u8) {
        self.f = (self.f & 0b0111_1111) | ((value & 0b0000_0001) << 7)
    }
    /// Get the zero flag `z` from the `f` register (pos 7).
    fn get_flag_z(&self) -> bool {
        self.f & 0x80 > 0
    }
    /// Set the subtraction flag `z` in the `f` register (pos 6).
    /// The least significant bit of `value` is used.
    fn set_flag_n(&mut self, value: u8) {
        self.f = (self.f & 0b1011_1111) | ((value & 0b0000_0001) << 6)
    }
    /// Get the subtraction flag `n` from the `f` register (pos 6).
    fn get_flag_n(&self) -> bool {
        self.f & 0x40 > 0
    }
    /// Set the half-carry flag `h` in the `f` register (pos 5).
    /// The least significant bit of `value` is used.
    fn set_flag_h(&mut self, value: u8) {
        self.f = (self.f & 0b1101_1111) | ((value & 0b0000_0001) << 5)
    }
    /// Get the half-carry flag `h` from the `f` register (pos 5).
    fn get_flag_h(&self) -> bool {
        self.f & 0x20 > 0
    }
    /// Set the carry flag `c` in the `f` register (pos 4).
    /// The least significant bit of `value` is used.
    fn set_flag_c(&mut self, value: u8) {
        self.f = (self.f & 0b1110_1111) | ((value & 0b0000_0001) << 4)
    }
    /// Get the carry flag `c` from the `f` register (pos 4).
    fn get_flag_c(&self) -> bool {
        self.f & 0x10 > 0
    }
}
