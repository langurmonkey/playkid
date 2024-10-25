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
    /// Flags register. See `Flag` below.
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

/// Represents the masks to access the flags in register `f`.
#[repr(u8)]
enum Flag {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

// Methods to access and set values of registers.
impl Registers {
    pub fn new() -> Self {
        // Initialize registers as per DMG.
        Registers {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: 0xB0,
            h: 0x01,
            l: 0x4D,
            // Points to the beginning of the stack, at 0xFFFE.
            sp: 0xFFFE,
            // We start at the 0x0100 location, which is te
            // begin code execution point.
            pc: 0x100,
        }
    }
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

    /// Private function to set the state of a flag.
    fn set_flag(&mut self, flag: Flag, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & !(flag as u8)) | (v << 7)
    }
    /// Private function to get the state of a flag.
    fn get_flag(&self, flag: Flag) -> bool {
        self.f & (flag as u8) > 0
    }
    /// Set the zero flag `z`.
    pub fn z(&mut self, value: bool) {
        self.set_flag(Flag::Z, value);
    }
    /// Get the zero flag `z` from the `f` register (pos 7).
    pub fn get_z(&self) -> bool {
        self.get_flag(Flag::Z)
    }
    /// Set the subtraction flag `n`.
    pub fn n(&mut self, value: bool) {
        self.set_flag(Flag::N, value);
    }
    /// Get the subtraction flag `n` from the `f` register (pos 6).
    pub fn get_n(&self) -> bool {
        self.get_flag(Flag::N)
    }
    /// Set the half-carry flag `h`.
    pub fn h(&mut self, value: bool) {
        self.set_flag(Flag::H, value);
    }
    /// Get the half-carry flag `h` from the `f` register (pos 5).
    pub fn get_h(&self) -> bool {
        self.get_flag(Flag::H)
    }
    /// Set the carry flag `c`.
    pub fn c(&mut self, value: bool) {
        self.set_flag(Flag::C, value);
    }
    /// Get the carry flag `c` from the `f` register (pos 4).
    pub fn get_c(&self) -> bool {
        self.get_flag(Flag::C)
    }
}
