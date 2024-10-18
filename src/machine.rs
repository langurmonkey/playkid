use crate::constants;
use crate::instruction;

use instruction::{Instruction, CC, R16, R16EXT, R16LD, R8, TGT3};

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
    fn get_hl_plus(&mut self) -> u16 {
        let result = self.get_hl();
        self.set_hl(result + 1);
        result
    }
    fn get_hl_minus(&mut self) -> u16 {
        let result = self.get_hl();
        self.set_hl(result - 1);
        result
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
    /// Set the zero flag `z`.
    fn z(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b0111_1111) | (v << 7)
    }
    /// Get the zero flag `z` from the `f` register (pos 7).
    fn get_flag_z(&self) -> bool {
        self.f & 0x80 > 0
    }
    /// Set the subtraction flag `n` in the `f` register (pos 6).
    /// The least significant bit of `value` is used.
    fn set_flag_n(&mut self, value: u8) {
        self.f = (self.f & 0b1011_1111) | ((value & 0b0000_0001) << 6)
    }
    /// Set the subtraction flag `n`.
    fn n(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b1011_1111) | (v << 6)
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
    /// Set the half-carry flag `h`.
    fn h(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b1101_1111) | (v << 5)
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
    /// Set the carry flag `c`.
    fn c(&mut self, value: bool) {
        let v: u8 = if value { 1 } else { 0 };
        self.f = (self.f & 0b1110_1111) | (v << 4)
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
    fn read16(&self, address: u16) -> u16 {
        (self.read(address) as u16) | ((self.read(address + 1) as u16) << 8)
    }
    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
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
    /// TODO: implement this.
    fn halt(&self) {}

    /// TODO: implement this.
    fn stop(&self) {}

    /// Execute a single instruction, and returns the number of cycles it takes.
    fn execute(&mut self, instr: Instruction) -> u8 {
        match instr {
            // NOP: do nothing
            Instruction::NOP() => 1,
            // STOP
            Instruction::STOP() => {
                self.stop();
                1
            }
            // HALT
            Instruction::HALT() => {
                self.halt();
                1
            }

            // LD r16
            Instruction::LD16(r16) => match r16 {
                R16::BC => {
                    let nw = self.read16();
                    self.registers.set_bc(nw);
                    3
                }
                R16::DE => {
                    let nw = self.read16();
                    self.registers.set_de(nw);
                    3
                }
                R16::HL => {
                    let nw = self.read16();
                    self.registers.set_hl(nw);
                    3
                }
                R16::SP => {
                    self.registers.sp = self.read16();
                    3
                }
            },
            // LD r8,r8
            Instruction::LDcp(r8_0, r8_1) => match r8_0 {
                R8::B => match r8_1 {
                    R8::B => 1,
                    R8::C => {
                        self.registers.b = self.registers.c;
                        1
                    }
                    R8::D => {
                        self.registers.b = self.registers.d;
                        1
                    }
                    R8::E => {
                        self.registers.b = self.registers.e;
                        1
                    }
                    R8::H => {
                        self.registers.b = self.registers.h;
                        1
                    }
                    R8::L => {
                        self.registers.b = self.registers.l;
                        1
                    }
                    R8::HL => {
                        self.registers.b = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.b = self.registers.a;
                        1
                    }
                },
                R8::C => match r8_1 {
                    R8::B => {
                        self.registers.c = self.registers.b;
                        1
                    }
                    R8::C => 1,
                    R8::D => {
                        self.registers.c = self.registers.d;
                        1
                    }
                    R8::E => {
                        self.registers.c = self.registers.e;
                        1
                    }
                    R8::H => {
                        self.registers.c = self.registers.h;
                        1
                    }
                    R8::L => {
                        self.registers.c = self.registers.l;
                        1
                    }
                    R8::HL => {
                        self.registers.c = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.c = self.registers.a;
                        1
                    }
                },
                R8::D => match r8_1 {
                    R8::B => {
                        self.registers.d = self.registers.b;
                        1
                    }
                    R8::C => {
                        self.registers.d = self.registers.c;
                        1
                    }
                    R8::D => 1,
                    R8::E => {
                        self.registers.d = self.registers.e;
                        1
                    }
                    R8::H => {
                        self.registers.d = self.registers.h;
                        1
                    }
                    R8::L => {
                        self.registers.d = self.registers.l;
                        1
                    }
                    R8::HL => {
                        self.registers.d = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.d = self.registers.a;
                        1
                    }
                },
                R8::E => match r8_1 {
                    R8::B => {
                        self.registers.e = self.registers.b;
                        1
                    }
                    R8::C => {
                        self.registers.e = self.registers.c;
                        1
                    }
                    R8::D => {
                        self.registers.e = self.registers.d;
                        1
                    }
                    R8::E => 1,
                    R8::H => {
                        self.registers.e = self.registers.h;
                        1
                    }
                    R8::L => {
                        self.registers.e = self.registers.l;
                        1
                    }
                    R8::HL => {
                        self.registers.e = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.e = self.registers.a;
                        1
                    }
                },
                R8::H => match r8_1 {
                    R8::B => {
                        self.registers.h = self.registers.b;
                        1
                    }
                    R8::C => {
                        self.registers.h = self.registers.c;
                        1
                    }
                    R8::D => {
                        self.registers.h = self.registers.d;
                        1
                    }
                    R8::E => {
                        self.registers.h = self.registers.e;
                        1
                    }
                    R8::H => 1,
                    R8::L => {
                        self.registers.h = self.registers.l;
                        1
                    }
                    R8::HL => {
                        self.registers.h = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.h = self.registers.a;
                        1
                    }
                },
                R8::L => match r8_1 {
                    R8::B => {
                        self.registers.l = self.registers.b;
                        1
                    }
                    R8::C => {
                        self.registers.l = self.registers.c;
                        1
                    }
                    R8::D => {
                        self.registers.l = self.registers.d;
                        1
                    }
                    R8::E => {
                        self.registers.l = self.registers.e;
                        1
                    }
                    R8::H => {
                        self.registers.l = self.registers.h;
                        1
                    }
                    R8::L => 1,
                    R8::HL => {
                        self.registers.l = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.l = self.registers.a;
                        1
                    }
                },
                R8::HL => match r8_1 {
                    R8::B => {
                        self.memory.write(self.registers.get_hl(), self.registers.b);
                        2
                    }
                    R8::C => {
                        self.memory.write(self.registers.get_hl(), self.registers.c);
                        2
                    }
                    R8::D => {
                        self.memory.write(self.registers.get_hl(), self.registers.d);
                        2
                    }
                    R8::E => {
                        self.memory.write(self.registers.get_hl(), self.registers.e);
                        2
                    }
                    R8::H => {
                        self.memory.write(self.registers.get_hl(), self.registers.h);
                        2
                    }
                    R8::L => {
                        self.memory.write(self.registers.get_hl(), self.registers.l);
                        2
                    }
                    R8::HL => 1,
                    R8::A => {
                        self.memory.write(self.registers.get_hl(), self.registers.a);
                        2
                    }
                },
                R8::A => match r8_1 {
                    R8::B => {
                        self.registers.a = self.registers.b;
                        1
                    }
                    R8::C => {
                        self.registers.a = self.registers.c;
                        1
                    }
                    R8::D => {
                        self.registers.a = self.registers.d;
                        1
                    }
                    R8::E => {
                        self.registers.a = self.registers.e;
                        1
                    }
                    R8::H => {
                        self.registers.a = self.registers.h;
                        1
                    }
                    R8::L => {
                        self.registers.a = self.registers.l;
                        1
                    }
                    R8::HL => {
                        self.registers.a = self.memory.read(self.registers.get_hl());
                        2
                    }
                    R8::A => 1,
                },
            },
            // LD r8
            Instruction::LD(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.read8();
                    2
                }
                R8::C => {
                    self.registers.c = self.read8();
                    2
                }
                R8::D => {
                    self.registers.d = self.read8();
                    2
                }
                R8::E => {
                    self.registers.e = self.read8();
                    2
                }
                R8::H => {
                    self.registers.h = self.read8();
                    2
                }
                R8::L => {
                    self.registers.l = self.read8();
                    2
                }
                R8::HL => {
                    let val = self.read8();
                    self.memory.write(self.registers.get_hl(), val);
                    3
                }
                R8::A => {
                    self.registers.a = self.read8();
                    2
                }
            },

            // LD x, A
            Instruction::LDfromA(r16ld) => match r16ld {
                R16LD::BC => {
                    self.memory.write(self.registers.get_bc(), self.registers.a);
                    2
                }
                R16LD::DE => {
                    self.memory.write(self.registers.get_de(), self.registers.a);
                    2
                }
                R16LD::HLp => {
                    self.memory
                        .write(self.registers.get_hl_plus(), self.registers.a);
                    2
                }
                R16LD::HLm => {
                    self.memory
                        .write(self.registers.get_hl_minus(), self.registers.a);
                    2
                }
                R16LD::A8 => {
                    let val = 0xFF00 | (self.read8() as u16);
                    self.memory.write(val, self.registers.a);
                    3
                }
                R16LD::C => {
                    self.memory
                        .write(0xFF00 | (self.registers.c as u16), self.registers.a);
                    2
                }
                R16LD::A16 => {
                    let val = self.read16();
                    self.memory.write(val, self.registers.a);
                    4
                }
            },
            // LD A, x
            Instruction::LDtoA(r16ld) => match r16ld {
                R16LD::BC => {
                    self.registers.a = self.memory.read(self.registers.get_bc());
                    2
                }
                R16LD::DE => {
                    self.registers.a = self.memory.read(self.registers.get_de());
                    2
                }
                R16LD::HLp => {
                    self.registers.a = self.memory.read(self.registers.get_hl_plus());
                    2
                }
                R16LD::HLm => {
                    self.registers.a = self.memory.read(self.registers.get_hl_minus());
                    2
                }
                R16LD::A8 => {
                    let val = 0xFF00 | (self.read8() as u16);
                    self.registers.a = self.memory.read(val);
                    3
                }
                R16LD::C => {
                    self.registers.a = self.memory.read(0xFF00 | (self.registers.c as u16));
                    2
                }
                R16LD::A16 => {
                    let val = self.read16();
                    self.registers.a = self.memory.read(val);
                    4
                }
            },

            // LD x, SP
            Instruction::LDfromSP() => {
                let val = self.add16imm(self.registers.sp);
                self.registers.set_hl(val);
                3
            }
            // LD SP, x
            Instruction::LDtoSP() => {
                self.registers.sp = self.registers.get_hl();
                2
            }
            // ADD SP, s8
            Instruction::ADDSP() => {
                self.registers.pc = self.add16imm(self.registers.sp);
                4
            }

            // ADD HL, r16
            Instruction::ADD16(r16) => match r16 {
                R16::BC => {
                    self.add16(self.registers.get_bc());
                    2
                }
                R16::DE => {
                    self.add16(self.registers.get_de());
                    2
                }
                R16::HL => {
                    self.add16(self.registers.get_hl());
                    2
                }
                R16::SP => {
                    self.add16(self.registers.sp);
                    2
                }
            },

            // ADD a, r8
            Instruction::ADD(r8) => match r8 {
                R8::A => {
                    self.add(self.registers.a, false);
                    1
                }
                R8::B => {
                    self.add(self.registers.b, false);
                    1
                }
                R8::C => {
                    self.add(self.registers.c, false);
                    1
                }
                R8::D => {
                    self.add(self.registers.d, false);
                    1
                }
                R8::E => {
                    self.add(self.registers.e, false);
                    1
                }
                R8::H => {
                    self.add(self.registers.h, false);
                    1
                }
                R8::L => {
                    self.add(self.registers.l, false);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.add(val, false);
                    2
                }
            },
            // ADC a, r8
            Instruction::ADC(r8) => match r8 {
                R8::A => {
                    self.add(self.registers.a, true);
                    1
                }
                R8::B => {
                    self.add(self.registers.b, true);
                    1
                }
                R8::C => {
                    self.add(self.registers.c, true);
                    1
                }
                R8::D => {
                    self.add(self.registers.d, true);
                    1
                }
                R8::E => {
                    self.add(self.registers.e, true);
                    1
                }
                R8::H => {
                    self.add(self.registers.h, true);
                    1
                }
                R8::L => {
                    self.add(self.registers.l, true);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.add(val, false);
                    2
                }
            },
            // SUB a, r8
            Instruction::SUB(r8) => match r8 {
                R8::A => {
                    self.sub(self.registers.a, false);
                    1
                }
                R8::B => {
                    self.sub(self.registers.b, false);
                    1
                }
                R8::C => {
                    self.sub(self.registers.c, false);
                    1
                }
                R8::D => {
                    self.sub(self.registers.d, false);
                    1
                }
                R8::E => {
                    self.sub(self.registers.e, false);
                    1
                }
                R8::H => {
                    self.sub(self.registers.h, false);
                    1
                }
                R8::L => {
                    self.sub(self.registers.l, false);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.sub(val, false);
                    2
                }
            },
            // SBC a, r8
            Instruction::SBC(r8) => match r8 {
                R8::A => {
                    self.sub(self.registers.a, true);
                    1
                }
                R8::B => {
                    self.sub(self.registers.b, true);
                    1
                }
                R8::C => {
                    self.sub(self.registers.c, true);
                    1
                }
                R8::D => {
                    self.sub(self.registers.d, true);
                    1
                }
                R8::E => {
                    self.sub(self.registers.e, true);
                    1
                }
                R8::H => {
                    self.sub(self.registers.h, true);
                    1
                }
                R8::L => {
                    self.sub(self.registers.l, true);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.sub(val, false);
                    2
                }
            },
            // AND a, r8
            Instruction::AND(r8) => match r8 {
                R8::A => {
                    self.and(self.registers.a);
                    1
                }
                R8::B => {
                    self.and(self.registers.b);
                    1
                }
                R8::C => {
                    self.and(self.registers.c);
                    1
                }
                R8::D => {
                    self.and(self.registers.d);
                    1
                }
                R8::E => {
                    self.and(self.registers.e);
                    1
                }
                R8::H => {
                    self.and(self.registers.h);
                    1
                }
                R8::L => {
                    self.and(self.registers.l);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.and(val);
                    2
                }
            },
            // XOR a, r8
            Instruction::XOR(r8) => match r8 {
                R8::A => {
                    self.xor(self.registers.a);
                    1
                }
                R8::B => {
                    self.xor(self.registers.b);
                    1
                }
                R8::C => {
                    self.xor(self.registers.c);
                    1
                }
                R8::D => {
                    self.xor(self.registers.d);
                    1
                }
                R8::E => {
                    self.xor(self.registers.e);
                    1
                }
                R8::H => {
                    self.xor(self.registers.h);
                    1
                }
                R8::L => {
                    self.xor(self.registers.l);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.xor(val);
                    2
                }
            },
            // OR a, r8
            Instruction::OR(r8) => match r8 {
                R8::A => {
                    self.or(self.registers.a);
                    1
                }
                R8::B => {
                    self.or(self.registers.b);
                    1
                }
                R8::C => {
                    self.or(self.registers.c);
                    1
                }
                R8::D => {
                    self.or(self.registers.d);
                    1
                }
                R8::E => {
                    self.or(self.registers.e);
                    1
                }
                R8::H => {
                    self.or(self.registers.h);
                    1
                }
                R8::L => {
                    self.or(self.registers.l);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.or(val);
                    2
                }
            },
            // CP a, r8
            Instruction::CP(r8) => match r8 {
                R8::A => {
                    self.cp(self.registers.a);
                    1
                }
                R8::B => {
                    self.cp(self.registers.b);
                    1
                }
                R8::C => {
                    self.cp(self.registers.c);
                    1
                }
                R8::D => {
                    self.cp(self.registers.d);
                    1
                }
                R8::E => {
                    self.cp(self.registers.e);
                    1
                }
                R8::H => {
                    self.cp(self.registers.h);
                    1
                }
                R8::L => {
                    self.cp(self.registers.l);
                    1
                }
                R8::HL => {
                    let val = self.memory.read(self.registers.get_hl());
                    self.cp(val);
                    2
                }
            },
            // ADD a, d8
            Instruction::ADDimm() => {
                let val = self.read8();
                self.add(val, false);
                2
            }
            // ADC a, d8
            Instruction::ADCimm() => {
                let val = self.read8();
                self.add(val, true);
                2
            }
            // SUB a, d8
            Instruction::SUBimm() => {
                let val = self.read8();
                self.sub(val, false);
                2
            }
            // SBC a, d8
            Instruction::SBCimm() => {
                let val = self.read8();
                self.sub(val, true);
                2
            }
            // AND a, d8
            Instruction::ANDimm() => {
                let val = self.read8();
                self.and(val);
                2
            }
            // XOR a, d8
            Instruction::XORimm() => {
                let val = self.read8();
                self.xor(val);
                2
            }
            // OR a, d8
            Instruction::ORimm() => {
                let val = self.read8();
                self.or(val);
                2
            }
            // CP a, d8
            Instruction::CPimm() => {
                let val = self.read8();
                self.cp(val);
                2
            }

            // JP HL
            Instruction::JPHL() => {
                self.registers.pc = self.registers.get_hl();
                1
            }
            // JP cond, a16
            Instruction::JP(cc) => match cc {
                CC::NONE => {
                    self.jp();
                    4
                }
                CC::NZ => {
                    if !self.registers.get_flag_z() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::Z => {
                    if self.registers.get_flag_z() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::NC => {
                    if !self.registers.get_flag_c() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::C => {
                    if self.registers.get_flag_c() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
            },
            // JR cond, a16
            Instruction::JR(cc) => match cc {
                CC::NONE => {
                    self.jr();
                    3
                }
                CC::NZ => {
                    if !self.registers.get_flag_z() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
                CC::Z => {
                    if self.registers.get_flag_z() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
                CC::NC => {
                    if !self.registers.get_flag_c() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
                CC::C => {
                    if self.registers.get_flag_c() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
            },

            // INC r16
            Instruction::INC16(r16) => match r16 {
                R16::BC => {
                    self.registers
                        .set_bc(self.registers.get_bc().wrapping_add(1));
                    2
                }
                R16::DE => {
                    self.registers
                        .set_de(self.registers.get_de().wrapping_add(1));
                    2
                }
                R16::HL => {
                    self.registers
                        .set_hl(self.registers.get_hl().wrapping_add(1));
                    2
                }
                R16::SP => {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    2
                }
            },
            // DEC r16
            Instruction::DEC16(r16) => match r16 {
                R16::BC => {
                    self.registers
                        .set_bc(self.registers.get_bc().wrapping_sub(1));
                    2
                }
                R16::DE => {
                    self.registers
                        .set_de(self.registers.get_de().wrapping_sub(1));
                    2
                }
                R16::HL => {
                    self.registers
                        .set_hl(self.registers.get_hl().wrapping_sub(1));
                    2
                }
                R16::SP => {
                    self.registers.sp = self.registers.sp.wrapping_sub(1);
                    2
                }
            },
            // INC r8
            Instruction::INC(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.inc(self.registers.b);
                    1
                }
                R8::C => {
                    self.registers.c = self.inc(self.registers.c);
                    1
                }
                R8::D => {
                    self.registers.d = self.inc(self.registers.d);
                    1
                }
                R8::E => {
                    self.registers.e = self.inc(self.registers.e);
                    1
                }
                R8::H => {
                    self.registers.h = self.inc(self.registers.h);
                    1
                }
                R8::L => {
                    self.registers.l = self.inc(self.registers.l);
                    1
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val_inc = self.inc(self.memory.read(hl));
                    self.memory.write(hl, val_inc);
                    3
                }
                R8::A => {
                    self.registers.a = self.inc(self.registers.a);
                    1
                }
            },
            // DEC r8
            Instruction::DEC(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.dec(self.registers.b);
                    1
                }
                R8::C => {
                    self.registers.c = self.dec(self.registers.c);
                    1
                }
                R8::D => {
                    self.registers.d = self.dec(self.registers.d);
                    1
                }
                R8::E => {
                    self.registers.e = self.dec(self.registers.e);
                    1
                }
                R8::H => {
                    self.registers.h = self.dec(self.registers.h);
                    1
                }
                R8::L => {
                    self.registers.l = self.dec(self.registers.l);
                    1
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val_dec = self.dec(self.memory.read(hl));
                    self.memory.write(hl, val_dec);
                    3
                }
                R8::A => {
                    self.registers.a = self.inc(self.registers.a);
                    1
                }
            },

            // RLCA
            Instruction::RLCA() => {
                self.registers.a = self.rlc(self.registers.a);
                self.registers.z(false);
                1
            }
            // RRCA
            Instruction::RRCA() => {
                self.registers.a = self.rrc(self.registers.a);
                self.registers.z(false);
                1
            }

            // RLA
            Instruction::RLA() => {
                self.registers.a = self.rl(self.registers.a);
                self.registers.z(false);
                1
            }
            // RRA
            Instruction::RRA() => {
                self.registers.a = self.rr(self.registers.a);
                self.registers.z(false);
                1
            }

            // DAA
            Instruction::DAA() => {
                // Apply corrections after addition or subtraction of two BCD numbers, whose result
                // is in `a`, and goes to `a`.
                // What we do:
                // - If addition, add 6 to each digit > 9, or if (half-)carry.
                // - If subtraction, subtract 6 from each digit > 9, or if (half-)carry.
                let mut a = self.registers.a;
                let c = self.registers.get_flag_c();
                let h = self.registers.get_flag_h();
                let n = self.registers.get_flag_n();

                if !n {
                    // After addition.
                    if c || a > 0x99 {
                        a += 0x60;
                        self.registers.c(true);
                    }
                    if h || (a & 0x0F) > 0x09 {
                        a += 0x6;
                    }
                } else {
                    // After subtraction.
                    if c {
                        a -= 0x60;
                    }
                    if h {
                        a -= 0x6;
                    }
                }
                self.registers.z(a == 0);
                self.registers.h(false);
                self.registers.a = a;
                1
            }
            // SCF
            Instruction::SCF() => {
                self.registers.c(true);
                self.registers.h(true);
                self.registers.n(true);
                1
            }
            // CPL
            Instruction::CPL() => {
                // Bitwise not of `a`.
                self.registers.a = !self.registers.a;
                self.registers.h(true);
                self.registers.n(true);
                1
            }
            // CCF
            Instruction::CCF() => {
                // Flip carry flag.
                self.registers.c(!self.registers.get_flag_c());
                self.registers.h(false);
                self.registers.n(false);
                1
            }

            // RET

            // POP

            // PUSH

            // CALL

            // RST

            // DI
            // EI
            _ => {
                // TODO: More instructions.
                0
            }
        }
    }

    /// Main loop of the machine.
    fn cycle(&mut self) {
        // Fetch next instruction, and parse it.
        let instruction_byte = self.memory.read(self.registers.pc);
        let instruction = Instruction::from_byte(instruction_byte);
    }

    /// Reads the next byte in memory at the location of `pc`, and
    /// increments `pc`.
    fn read8(&mut self) -> u8 {
        let result = self.memory.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        result
    }

    /// Reads the next two bytes in memory at the location of `pc`, and
    /// increments `pc` twice.
    fn read16(&mut self) -> u16 {
        let result = self.memory.read16(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(2);
        result
    }

    fn jp(&mut self) {
        self.registers.pc = self.read16();
    }

    fn jr(&mut self) {
        let j = self.read8() as i8;
        self.registers.pc = ((self.registers.pc as i32) + (j as i32)) as u16;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.registers.z(result == 0);
        self.registers.h((value & 0x0F) + 1 > 0x0F);
        self.registers.n(false);
        result
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.registers.z(result == 0);
        self.registers.h((value & 0x0F) == 0);
        self.registers.n(true);
        result
    }

    fn rotate_flags(&mut self, result: u8, carry: bool) {
        self.registers.z(result == 0);
        self.registers.c(carry);
        self.registers.h(false);
        self.registers.n(false);
    }

    fn rlc(&mut self, val: u8) -> u8 {
        let carry = val & 0x80 > 0;
        let result = (val << 1) | (if carry { 1 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    fn rl(&mut self, val: u8) -> u8 {
        let carry = val & 0x80 > 0;
        let result = (val << 1) | (if self.registers.get_flag_c() { 1 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    fn rrc(&mut self, val: u8) -> u8 {
        let carry = val & 0x01 > 0;
        let result = (val >> 1) | (if carry { 0x80 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    fn rr(&mut self, val: u8) -> u8 {
        let carry = val & 0x01 > 0;
        let result = (val >> 1) | (if self.registers.get_flag_c() { 0x80 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    /// Adds the given 16-bit value to `hl`.
    fn add16(&mut self, value: u16) {
        let hl = self.registers.get_hl();
        // Actual addition.
        let result = hl.wrapping_add(value);
        // Update zero flag.
        self.registers.z(result == 0);
        // Update subtraction flag.
        self.registers.n(false);
        // Update carry flag (overflow from bit 15).
        self.registers.c(hl > 0xFFFF - value);
        // Update half-carry flag (overflow from bit 11).
        self.registers.h((hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);

        // Result -> hl.
        self.registers.set_hl(result);
    }

    /// Adds the given value to the next immediate 16-bit signed number, and update flags.
    fn add16imm(&mut self, value: u16) -> u16 {
        let v: u16 = self.read8() as i8 as i16 as u16;
        self.registers.n(false);
        self.registers.z(false);
        self.registers.h((value & 0x000F) + (v & 0x000F) > 0x000F);
        self.registers.c((value & 0x00FF) + (v & 0x00FF) > 0x00FF);
        value.wrapping_add(v)
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
        self.registers.z(result == 0);
        // Update subtraction flag.
        self.registers.n(false);
        // Update carry flag.
        self.registers.c(overflow);
        // Update half-carry flag. The half-carry is 1 if the addition of the
        // lower nibbles of a and target overflows.
        self.registers
            .h((a & 0x0F) + (result & 0x0F) + carry > 0x0F);

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
        self.registers.z(result == 0);
        // Update subtraction flag.
        self.registers.n(false);
        // Update carry flag if borrow (value+carry > a).
        self.registers
            .c((value as u16) + (carry as u16) > (a as u16));
        // Update half-carry flag. Set if borrow from bit 4.
        self.registers.h((a & 0x0F) < (value & 0x0F) + carry);

        // Result -> a.
        self.registers.a = result;
    }

    fn and(&mut self, value: u8) {
        let result = self.registers.a & value;
        self.registers.z(result == 0);
        self.registers.n(false);
        self.registers.c(false);
        self.registers.h(true);
        // Result -> a.
        self.registers.a = result;
    }

    fn xor(&mut self, value: u8) {
        let result = self.registers.a ^ value;
        self.registers.z(result == 0);
        self.registers.n(false);
        self.registers.c(false);
        self.registers.h(false);
        // Result -> a.
        self.registers.a = result;
    }

    fn or(&mut self, value: u8) {
        let result = self.registers.a | value;
        self.registers.z(result == 0);
        self.registers.n(false);
        self.registers.c(false);
        self.registers.h(false);
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
