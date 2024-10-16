use crate::constants;
use crate::instruction;

use instruction::{Instruction, R8};

/// # Registers
/// We have 7 1-bit registers (`a`, `b`, `c`, `d`, `e`, `h`, `l`) which can be accessed individually,
/// or together as 16 bits, in the combinations `af`, `bc`, `de` and `hl`.
/// We also have the flags register, `f`, whose 4 most significant bits are the flags
/// zero `z`, subtraction `n`, half-carry `h` and carry `c`.
/// Additionally, we have two 16-bit special registers, the stack pointer `sp`, and
/// the program counter `pc`.
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

/// # Memory
/// The Game Boy uses a 2-byte address space (0x0000 to 0xFFFF) to map the different
/// types of memory (RAM, VRAM, Cartridge memory, etc.)
/// # VRAM
/// A **memory bank** contains 384 tiles, or 3 tile blocks, so 6 KiB of tile data.
/// After that, it  has two maps of 1024 bytes each.
/// In total, a bank has 8 KiB of memory.
///
/// - A **tile** has 8x8 pixels, with a color depth of 2 bpp. Each tile is 16 bytes.
///   Tiles in a bank are typically grouped into blocks.
/// - A **tile block** contains 128 tiles of 16 bytes each, so 2048 bytes.
/// - A **map** contains 32x32=1024 bytes.
pub struct Memory {
    pub data: [u8; constants::MEM_SIZE],
}

impl Memory {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

/// This is our machine, which contains the registers and the memory.
struct GameBoy {
    /// Our registers.
    registers: Registers,
    /// The main memory.
    memory: Memory,
}

impl GameBoy {
    /// Execute a single instruction.
    fn execute(&mut self, instr: Instruction) {
        match instr {
            // ADD 8-bit
            Instruction::ADD(r8) => match r8 {
                R8::A => {
                    self.add(self.registers.a, false);
                }
                R8::B => {
                    self.add(self.registers.b, false);
                }
                R8::C => {
                    self.add(self.registers.c, false);
                }
                R8::D => {
                    self.add(self.registers.d, false);
                }
                R8::E => {
                    self.add(self.registers.e, false);
                }
                R8::H => {
                    self.add(self.registers.h, false);
                }
                R8::L => {
                    self.add(self.registers.l, false);
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.add(val, false);
                }
            },
            // ADC 8-bit
            Instruction::ADC(r8) => match r8 {
                R8::A => {
                    self.add(self.registers.a, true);
                }
                R8::B => {
                    self.add(self.registers.b, true);
                }
                R8::C => {
                    self.add(self.registers.c, true);
                }
                R8::D => {
                    self.add(self.registers.d, true);
                }
                R8::E => {
                    self.add(self.registers.e, true);
                }
                R8::H => {
                    self.add(self.registers.h, true);
                }
                R8::L => {
                    self.add(self.registers.l, true);
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.add(val, false);
                }
            },
            // SUB 8-bit
            Instruction::SUB(r8) => match r8 {
                R8::A => {
                    self.sub(self.registers.a, false);
                }
                R8::B => {
                    self.sub(self.registers.b, false);
                }
                R8::C => {
                    self.sub(self.registers.c, false);
                }
                R8::D => {
                    self.sub(self.registers.d, false);
                }
                R8::E => {
                    self.sub(self.registers.e, false);
                }
                R8::H => {
                    self.sub(self.registers.h, false);
                }
                R8::L => {
                    self.sub(self.registers.l, false);
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.sub(val, false);
                }
            },
            // SBC 8-bit
            Instruction::SBC(r8) => match r8 {
                R8::A => {
                    self.sub(self.registers.a, true);
                }
                R8::B => {
                    self.sub(self.registers.b, true);
                }
                R8::C => {
                    self.sub(self.registers.c, true);
                }
                R8::D => {
                    self.sub(self.registers.d, true);
                }
                R8::E => {
                    self.sub(self.registers.e, true);
                }
                R8::H => {
                    self.sub(self.registers.h, true);
                }
                R8::L => {
                    self.sub(self.registers.l, true);
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.sub(val, false);
                }
            },
            // AND 8-bit
            Instruction::AND(r8) => match r8 {
                R8::A => {
                    self.and(self.registers.a);
                }
                R8::B => {
                    self.and(self.registers.b);
                }
                R8::C => {
                    self.and(self.registers.c);
                }
                R8::D => {
                    self.and(self.registers.d);
                }
                R8::E => {
                    self.and(self.registers.e);
                }
                R8::H => {
                    self.and(self.registers.h);
                }
                R8::L => {
                    self.and(self.registers.l);
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.and(val);
                }
            },
            // XOR 8-bit
            Instruction::XOR(r8) => match r8 {
                R8::A => {
                    self.xor(self.registers.a);
                }
                R8::B => {
                    self.xor(self.registers.b);
                }
                R8::C => {
                    self.xor(self.registers.c);
                }
                R8::D => {
                    self.xor(self.registers.d);
                }
                R8::E => {
                    self.xor(self.registers.e);
                }
                R8::H => {
                    self.xor(self.registers.h);
                }
                R8::L => {
                    self.xor(self.registers.l);
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.xor(val);
                }
            },
            _ => {
                // TODO: More instructions.
            }
        }
    }

    /// Main loop of the machine.
    fn cycle(&mut self) {
        // Fetch next instruction, and parse it.
        let instruction_byte = self.memory.read(self.registers.pc);
        let instruction = Instruction::from_byte(instruction_byte);
    }

    /// Adds the given byte to the register `a` and updates the flags.
    fn add(&mut self, value: u8, use_carry: bool) {
        // Get carry if needed.
        let carry = if use_carry && self.registers.get_flag_c() {
            1
        } else {
            0
        };
        let a = self.registers.a;
        // Actual addition.
        let result = a.wrapping_add(value).wrapping_add(carry);
        // Compute overflow.
        let overflow = (a as u16) + (value as u16) + (carry as u16) > 0xFF;
        // Update zero flag.
        self.registers.set_flag_z((result == 0) as u8);
        // Update subtraction flag.
        self.registers.set_flag_n(0);
        // Update carry flag.
        self.registers.set_flag_c(overflow as u8);
        // Update half-carry flag. The half-carry is 1 if the addition of the
        // lower nibbles of a and target overflows.
        self.registers
            .set_flag_h(((a & 0x0F) + (result & 0x0F) + carry > 0x0F) as u8);

        // Result -> a.
        self.registers.a = result;
    }

    fn sub(&mut self, value: u8, use_carry: bool) {
        // Get carry if needed.
        let carry = if use_carry && self.registers.get_flag_c() {
            1
        } else {
            0
        };
        let a = self.registers.a;
        // Actual subtraction.
        let result = a.wrapping_sub(value).wrapping_sub(carry);
        //Update zero flag.
        self.registers.set_flag_z((result == 0) as u8);
        // Update subtraction flag.
        self.registers.set_flag_n(1);
        // Update carry flag if borrow (value+carry > a).
        self.registers
            .set_flag_c(((value as u16) + (carry as u16) > (a as u16)) as u8);
        // Update half-carry flag. Set if borrow from bit 4.
        self.registers
            .set_flag_h(((a & 0x0F) < (value & 0x0F) + carry) as u8);

        // Result -> a.
        self.registers.a = result;
    }

    fn and(&mut self, value: u8) {
        let result = self.registers.a & value;
        self.registers.set_flag_z((result == 0) as u8);
        self.registers.set_flag_n(0);
        self.registers.set_flag_c(0);
        self.registers.set_flag_h(1);
        // Result -> a.
        self.registers.a = result;
    }

    fn xor(&mut self, value: u8) {
        let result = self.registers.a ^ value;
        self.registers.set_flag_z((result == 0) as u8);
        self.registers.set_flag_n(0);
        self.registers.set_flag_c(0);
        self.registers.set_flag_h(0);
        // Result -> a.
        self.registers.a = result;
    }

    fn or(&mut self, value: u8) {
        let result = self.registers.a | value;
        self.registers.set_flag_z((result == 0) as u8);
        self.registers.set_flag_n(0);
        self.registers.set_flag_c(0);
        self.registers.set_flag_h(0);
        // Result -> a.
        self.registers.a = result;
    }
    fn cp(&mut self, value: u8) {
        let backup = self.registers.a;
        self.sub(value, false);
        // Do not store the value.
        self.registers.a = backup;
    }
}
