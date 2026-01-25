use crate::memory::Memory;
use crate::registers::Registers;
use std::fmt;

/// # Instructions
/// This enum contains all supported instructions.
/// Instructions that can act with both r8 and r16 registers are first named
/// by default in their r8 version. The r16 versions add a `16` at the end of the name.
#[derive(Debug)]
pub enum Instruction {
    // NoOp.
    NOP(),
    // Stop.
    STOP(),
    // Halt.
    HALT(),

    // Jump HL.
    JPHL(),
    // Jump.
    JP(CC),
    // Relative jumps.
    JR(CC),

    // Push.
    PUSH(R16EXT),
    // Pop.
    POP(R16EXT),

    // Load.
    LD(R8),
    LDcp(R8, R8),
    LD16(R16),
    LDfromA(R16LD),
    LDtoA(R16LD),
    LD16SP(),
    LDfromSP(),
    LDtoSP(),

    // Increment and decrement.
    INC(R8),
    DEC(R8),
    INC16(R16),
    DEC16(R16),

    // 8-bit arithmetic with registers and immediate bytes.
    ADD16(R16),
    ADD(R8),
    ADDimm(),
    ADC(R8),
    ADCimm(),
    SUB(R8),
    SUBimm(),
    SBC(R8),
    SBCimm(),
    AND(R8),
    ANDimm(),
    XOR(R8),
    XORimm(),
    OR(R8),
    ORimm(),
    CP(R8),
    CPimm(),
    ADDSP(),

    // Bit shifts.
    RLCA(),
    RRCA(),
    RLA(),
    RRA(),

    // Decimal adjust accumulator.
    DAA(),
    // Set carry flag.
    SCF(),
    // Complement accumulator.
    CPL(),
    // Complement carry flag.
    CCF(),

    // Calls.
    CALL(CC),
    // Call address vec.
    RST(TGT3),

    // Interrupts.
    DI(),
    EI(),

    // Return.
    RET(CC),
    // Return interrupt-service routine.
    RETI(),

    // 16-bit opcodes (below).
    // This virtual instruction leads to interpreting the next byte as some
    // of the opcodes immediately following.
    OPCODE16(),

    // RLC r8
    RLC(R8),
    // RRC r8
    RRC(R8),
    // RL r8
    RL(R8),
    // RR r8
    RR(R8),
    // SLA r8
    SLA(R8),
    // SRA r8
    SRA(R8),
    // SWAP r8
    SWAP(R8),
    // SRL r8
    SRL(R8),

    // BIT0 r8
    BIT0(R8),
    // BIT1 r8
    BIT1(R8),
    // BIT2 r8
    BIT2(R8),
    // BIT3 r8
    BIT3(R8),
    // BIT4 r8
    BIT4(R8),
    // BIT5 r8
    BIT5(R8),
    // BIT6 r8
    BIT6(R8),
    // BIT7 r8
    BIT7(R8),

    // RES0 r8
    RES0(R8),
    // RES1 r8
    RES1(R8),
    // RES2 r8
    RES2(R8),
    // RES3 r8
    RES3(R8),
    // RES4 r8
    RES4(R8),
    // RES5 r8
    RES5(R8),
    // RES6 r8
    RES6(R8),
    // RES7 r8
    RES7(R8),

    // SET0 r8
    SET0(R8),
    // SET1 r8
    SET1(R8),
    // SET2 r8
    SET2(R8),
    // SET3 r8
    SET3(R8),
    // SET4 r8
    SET4(R8),
    // SET5 r8
    SET5(R8),
    // SET6 r8
    SET6(R8),
    // SET7 r8
    SET7(R8),
}

/// Enumerates the r8 registers.
#[derive(Debug)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

/// Enumerates the r16 registers.
#[derive(Debug)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

/// Enumerates r16 registers for POP and PUSH, which contain AF.
#[derive(Debug)]
pub enum R16EXT {
    BC,
    DE,
    HL,
    AF,
}

/// Enumerates the R16 registers to be used in (some) load operations.
#[derive(Debug)]
pub enum R16LD {
    BC,
    DE,
    HLp,
    HLm,
    A8,
    C,
    A16,
}

/// Enumerates jump conditions, mostly flags.
#[derive(Debug)]
pub enum CC {
    NONE,
    NZ,
    Z,
    NC,
    C,
}

/// RST's target address, divided by 8.
#[derive(Debug)]
pub enum TGT3 {
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
}

impl Instruction {
    /// Construct an instruction from its byte representation.
    pub fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            // NOP
            0x00 => Some(Instruction::NOP()),
            // STOP
            0x10 => Some(Instruction::STOP()),

            // LD (r16)
            0x01 => Some(Instruction::LD16(R16::BC)),
            0x11 => Some(Instruction::LD16(R16::DE)),
            0x21 => Some(Instruction::LD16(R16::HL)),
            0x31 => Some(Instruction::LD16(R16::SP)),

            // LD (r8, r8)
            // to B
            0x40 => Some(Instruction::LDcp(R8::B, R8::B)),
            0x41 => Some(Instruction::LDcp(R8::B, R8::C)),
            0x42 => Some(Instruction::LDcp(R8::B, R8::D)),
            0x43 => Some(Instruction::LDcp(R8::B, R8::E)),
            0x44 => Some(Instruction::LDcp(R8::B, R8::H)),
            0x45 => Some(Instruction::LDcp(R8::B, R8::L)),
            0x46 => Some(Instruction::LDcp(R8::B, R8::HL)),
            0x47 => Some(Instruction::LDcp(R8::B, R8::A)),
            // to C
            0x48 => Some(Instruction::LDcp(R8::C, R8::B)),
            0x49 => Some(Instruction::LDcp(R8::C, R8::C)),
            0x4A => Some(Instruction::LDcp(R8::C, R8::D)),
            0x4B => Some(Instruction::LDcp(R8::C, R8::E)),
            0x4C => Some(Instruction::LDcp(R8::C, R8::H)),
            0x4D => Some(Instruction::LDcp(R8::C, R8::L)),
            0x4E => Some(Instruction::LDcp(R8::C, R8::HL)),
            0x4F => Some(Instruction::LDcp(R8::C, R8::A)),
            // to D
            0x50 => Some(Instruction::LDcp(R8::D, R8::B)),
            0x51 => Some(Instruction::LDcp(R8::D, R8::C)),
            0x52 => Some(Instruction::LDcp(R8::D, R8::D)),
            0x53 => Some(Instruction::LDcp(R8::D, R8::E)),
            0x54 => Some(Instruction::LDcp(R8::D, R8::H)),
            0x55 => Some(Instruction::LDcp(R8::D, R8::L)),
            0x56 => Some(Instruction::LDcp(R8::D, R8::HL)),
            0x57 => Some(Instruction::LDcp(R8::D, R8::A)),
            // to E
            0x58 => Some(Instruction::LDcp(R8::E, R8::B)),
            0x59 => Some(Instruction::LDcp(R8::E, R8::C)),
            0x5A => Some(Instruction::LDcp(R8::E, R8::D)),
            0x5B => Some(Instruction::LDcp(R8::E, R8::E)),
            0x5C => Some(Instruction::LDcp(R8::E, R8::H)),
            0x5D => Some(Instruction::LDcp(R8::E, R8::L)),
            0x5E => Some(Instruction::LDcp(R8::E, R8::HL)),
            0x5F => Some(Instruction::LDcp(R8::E, R8::A)),
            // to H
            0x60 => Some(Instruction::LDcp(R8::H, R8::B)),
            0x61 => Some(Instruction::LDcp(R8::H, R8::C)),
            0x62 => Some(Instruction::LDcp(R8::H, R8::D)),
            0x63 => Some(Instruction::LDcp(R8::H, R8::E)),
            0x64 => Some(Instruction::LDcp(R8::H, R8::H)),
            0x65 => Some(Instruction::LDcp(R8::H, R8::L)),
            0x66 => Some(Instruction::LDcp(R8::H, R8::HL)),
            0x67 => Some(Instruction::LDcp(R8::H, R8::A)),
            // to L
            0x68 => Some(Instruction::LDcp(R8::L, R8::B)),
            0x69 => Some(Instruction::LDcp(R8::L, R8::C)),
            0x6A => Some(Instruction::LDcp(R8::L, R8::D)),
            0x6B => Some(Instruction::LDcp(R8::L, R8::E)),
            0x6C => Some(Instruction::LDcp(R8::L, R8::H)),
            0x6D => Some(Instruction::LDcp(R8::L, R8::L)),
            0x6E => Some(Instruction::LDcp(R8::L, R8::HL)),
            0x6F => Some(Instruction::LDcp(R8::L, R8::A)),
            // to HL
            0x70 => Some(Instruction::LDcp(R8::HL, R8::B)),
            0x71 => Some(Instruction::LDcp(R8::HL, R8::C)),
            0x72 => Some(Instruction::LDcp(R8::HL, R8::D)),
            0x73 => Some(Instruction::LDcp(R8::HL, R8::E)),
            0x74 => Some(Instruction::LDcp(R8::HL, R8::H)),
            0x75 => Some(Instruction::LDcp(R8::HL, R8::L)),
            0x76 => Some(Instruction::HALT()),
            0x77 => Some(Instruction::LDcp(R8::HL, R8::A)),
            // to A
            0x78 => Some(Instruction::LDcp(R8::A, R8::B)),
            0x79 => Some(Instruction::LDcp(R8::A, R8::C)),
            0x7A => Some(Instruction::LDcp(R8::A, R8::D)),
            0x7B => Some(Instruction::LDcp(R8::A, R8::E)),
            0x7C => Some(Instruction::LDcp(R8::A, R8::H)),
            0x7D => Some(Instruction::LDcp(R8::A, R8::L)),
            0x7E => Some(Instruction::LDcp(R8::A, R8::HL)),
            0x7F => Some(Instruction::LDcp(R8::A, R8::A)),

            // LD (r8)
            0x06 => Some(Instruction::LD(R8::B)),
            0x0E => Some(Instruction::LD(R8::C)),
            0x16 => Some(Instruction::LD(R8::D)),
            0x1E => Some(Instruction::LD(R8::E)),
            0x26 => Some(Instruction::LD(R8::H)),
            0x2E => Some(Instruction::LD(R8::L)),
            0x36 => Some(Instruction::LD(R8::HL)),
            0x3E => Some(Instruction::LD(R8::A)),

            // LD x, A
            0x02 => Some(Instruction::LDfromA(R16LD::BC)),
            0x12 => Some(Instruction::LDfromA(R16LD::DE)),
            0x22 => Some(Instruction::LDfromA(R16LD::HLp)),
            0x32 => Some(Instruction::LDfromA(R16LD::HLm)),
            0xE0 => Some(Instruction::LDfromA(R16LD::A8)),
            0xE2 => Some(Instruction::LDfromA(R16LD::C)),
            0xEA => Some(Instruction::LDfromA(R16LD::A16)),

            // LD A, x
            0x0A => Some(Instruction::LDtoA(R16LD::BC)),
            0x1A => Some(Instruction::LDtoA(R16LD::DE)),
            0x2A => Some(Instruction::LDtoA(R16LD::HLp)),
            0x3A => Some(Instruction::LDtoA(R16LD::HLm)),
            0xF0 => Some(Instruction::LDtoA(R16LD::A8)),
            0xF2 => Some(Instruction::LDtoA(R16LD::C)),
            0xFA => Some(Instruction::LDtoA(R16LD::A16)),

            // LD (r16) SP
            0x08 => Some(Instruction::LD16SP()),
            // LD x, SP
            0xF8 => Some(Instruction::LDfromSP()),
            // LD SP, x
            0xF9 => Some(Instruction::LDtoSP()),
            // ADD SP, s8
            0xE8 => Some(Instruction::ADDSP()),

            // ADD HL, r16
            0x09 => Some(Instruction::ADD16(R16::BC)),
            0x19 => Some(Instruction::ADD16(R16::DE)),
            0x29 => Some(Instruction::ADD16(R16::HL)),
            0x39 => Some(Instruction::ADD16(R16::SP)),
            // ADD a, r8
            0x80 => Some(Instruction::ADD(R8::B)),
            0x81 => Some(Instruction::ADD(R8::C)),
            0x82 => Some(Instruction::ADD(R8::D)),
            0x83 => Some(Instruction::ADD(R8::E)),
            0x84 => Some(Instruction::ADD(R8::H)),
            0x85 => Some(Instruction::ADD(R8::L)),
            0x86 => Some(Instruction::ADD(R8::HL)),
            0x87 => Some(Instruction::ADD(R8::A)),
            // ADC a, r8
            0x88 => Some(Instruction::ADC(R8::B)),
            0x89 => Some(Instruction::ADC(R8::C)),
            0x8A => Some(Instruction::ADC(R8::D)),
            0x8B => Some(Instruction::ADC(R8::E)),
            0x8C => Some(Instruction::ADC(R8::H)),
            0x8D => Some(Instruction::ADC(R8::L)),
            0x8E => Some(Instruction::ADC(R8::HL)),
            0x8F => Some(Instruction::ADC(R8::A)),

            // SUB a, r8
            0x90 => Some(Instruction::SUB(R8::B)),
            0x91 => Some(Instruction::SUB(R8::C)),
            0x92 => Some(Instruction::SUB(R8::D)),
            0x93 => Some(Instruction::SUB(R8::E)),
            0x94 => Some(Instruction::SUB(R8::H)),
            0x95 => Some(Instruction::SUB(R8::L)),
            0x96 => Some(Instruction::SUB(R8::HL)),
            0x97 => Some(Instruction::SUB(R8::A)),
            // SBC a, r8
            0x98 => Some(Instruction::SBC(R8::B)),
            0x99 => Some(Instruction::SBC(R8::C)),
            0x9A => Some(Instruction::SBC(R8::D)),
            0x9B => Some(Instruction::SBC(R8::E)),
            0x9C => Some(Instruction::SBC(R8::H)),
            0x9D => Some(Instruction::SBC(R8::L)),
            0x9E => Some(Instruction::SBC(R8::HL)),
            0x9F => Some(Instruction::SBC(R8::A)),

            // AND a, r8
            0xA0 => Some(Instruction::AND(R8::B)),
            0xA1 => Some(Instruction::AND(R8::C)),
            0xA2 => Some(Instruction::AND(R8::D)),
            0xA3 => Some(Instruction::AND(R8::E)),
            0xA4 => Some(Instruction::AND(R8::H)),
            0xA5 => Some(Instruction::AND(R8::L)),
            0xA6 => Some(Instruction::AND(R8::HL)),
            0xA7 => Some(Instruction::AND(R8::A)),
            // XOR a, r8
            0xA8 => Some(Instruction::XOR(R8::B)),
            0xA9 => Some(Instruction::XOR(R8::C)),
            0xAA => Some(Instruction::XOR(R8::D)),
            0xAB => Some(Instruction::XOR(R8::E)),
            0xAC => Some(Instruction::XOR(R8::H)),
            0xAD => Some(Instruction::XOR(R8::L)),
            0xAE => Some(Instruction::XOR(R8::HL)),
            0xAF => Some(Instruction::XOR(R8::A)),
            // OR a, r8
            0xB0 => Some(Instruction::OR(R8::B)),
            0xB1 => Some(Instruction::OR(R8::C)),
            0xB2 => Some(Instruction::OR(R8::D)),
            0xB3 => Some(Instruction::OR(R8::E)),
            0xB4 => Some(Instruction::OR(R8::H)),
            0xB5 => Some(Instruction::OR(R8::L)),
            0xB6 => Some(Instruction::OR(R8::HL)),
            0xB7 => Some(Instruction::OR(R8::A)),
            // CP a, r8
            0xB8 => Some(Instruction::CP(R8::B)),
            0xB9 => Some(Instruction::CP(R8::C)),
            0xBA => Some(Instruction::CP(R8::D)),
            0xBB => Some(Instruction::CP(R8::E)),
            0xBC => Some(Instruction::CP(R8::H)),
            0xBD => Some(Instruction::CP(R8::L)),
            0xBE => Some(Instruction::CP(R8::HL)),
            0xBF => Some(Instruction::CP(R8::A)),

            // ADD,ADC,SUB,SBC,AND,XOR,OR,CP a, d8
            0xC6 => Some(Instruction::ADDimm()),
            0xCE => Some(Instruction::ADCimm()),
            0xD6 => Some(Instruction::SUBimm()),
            0xDE => Some(Instruction::SBCimm()),
            0xE6 => Some(Instruction::ANDimm()),
            0xEE => Some(Instruction::XORimm()),
            0xF6 => Some(Instruction::ORimm()),
            0xFE => Some(Instruction::CPimm()),

            // JP HL
            0xE9 => Some(Instruction::JPHL()),
            // JP cond, a16
            0xC3 => Some(Instruction::JP(CC::NONE)),
            0xC2 => Some(Instruction::JP(CC::NZ)),
            0xD2 => Some(Instruction::JP(CC::NC)),
            0xCA => Some(Instruction::JP(CC::Z)),
            0xDA => Some(Instruction::JP(CC::C)),
            // JR cond, a16
            0x18 => Some(Instruction::JR(CC::NONE)),
            0x20 => Some(Instruction::JR(CC::NZ)),
            0x28 => Some(Instruction::JR(CC::Z)),
            0x30 => Some(Instruction::JR(CC::NC)),
            0x38 => Some(Instruction::JR(CC::C)),

            // INC r16
            0x03 => Some(Instruction::INC16(R16::BC)),
            0x13 => Some(Instruction::INC16(R16::DE)),
            0x23 => Some(Instruction::INC16(R16::HL)),
            0x33 => Some(Instruction::INC16(R16::SP)),
            // DEC r16
            0x0B => Some(Instruction::DEC16(R16::BC)),
            0x1B => Some(Instruction::DEC16(R16::DE)),
            0x2B => Some(Instruction::DEC16(R16::HL)),
            0x3B => Some(Instruction::DEC16(R16::SP)),
            // INC r8
            0x04 => Some(Instruction::INC(R8::B)),
            0x0C => Some(Instruction::INC(R8::C)),
            0x14 => Some(Instruction::INC(R8::D)),
            0x1C => Some(Instruction::INC(R8::E)),
            0x24 => Some(Instruction::INC(R8::H)),
            0x2C => Some(Instruction::INC(R8::L)),
            0x34 => Some(Instruction::INC(R8::HL)),
            0x3C => Some(Instruction::INC(R8::A)),
            // DEC r8
            0x05 => Some(Instruction::DEC(R8::B)),
            0x0D => Some(Instruction::DEC(R8::C)),
            0x15 => Some(Instruction::DEC(R8::D)),
            0x1D => Some(Instruction::DEC(R8::E)),
            0x25 => Some(Instruction::DEC(R8::H)),
            0x2D => Some(Instruction::DEC(R8::L)),
            0x35 => Some(Instruction::DEC(R8::HL)),
            0x3D => Some(Instruction::DEC(R8::A)),

            // RRCA
            0x0F => Some(Instruction::RRCA()),
            // RRA
            0x1F => Some(Instruction::RRA()),
            // RLCA
            0x07 => Some(Instruction::RLCA()),
            // RLA
            0x17 => Some(Instruction::RLA()),

            // DAA
            0x27 => Some(Instruction::DAA()),
            // SCF
            0x37 => Some(Instruction::SCF()),
            // CPL
            0x2F => Some(Instruction::CPL()),
            // CCF
            0x3F => Some(Instruction::CCF()),

            // RET
            0xC0 => Some(Instruction::RET(CC::NZ)),
            0xD0 => Some(Instruction::RET(CC::NC)),
            0xC8 => Some(Instruction::RET(CC::Z)),
            0xD8 => Some(Instruction::RET(CC::C)),
            0xC9 => Some(Instruction::RET(CC::NONE)),
            0xD9 => Some(Instruction::RETI()),

            // POP
            0xC1 => Some(Instruction::POP(R16EXT::BC)),
            0xD1 => Some(Instruction::POP(R16EXT::DE)),
            0xE1 => Some(Instruction::POP(R16EXT::HL)),
            0xF1 => Some(Instruction::POP(R16EXT::AF)),

            // PUSH
            0xC5 => Some(Instruction::PUSH(R16EXT::BC)),
            0xD5 => Some(Instruction::PUSH(R16EXT::DE)),
            0xE5 => Some(Instruction::PUSH(R16EXT::HL)),
            0xF5 => Some(Instruction::PUSH(R16EXT::AF)),

            // CALL
            0xC4 => Some(Instruction::CALL(CC::NZ)),
            0xD4 => Some(Instruction::CALL(CC::NC)),
            0xCC => Some(Instruction::CALL(CC::Z)),
            0xDC => Some(Instruction::CALL(CC::C)),
            0xCD => Some(Instruction::CALL(CC::NONE)),

            // RST
            0xC7 => Some(Instruction::RST(TGT3::T0)),
            0xCF => Some(Instruction::RST(TGT3::T1)),
            0xD7 => Some(Instruction::RST(TGT3::T2)),
            0xDF => Some(Instruction::RST(TGT3::T3)),
            0xE7 => Some(Instruction::RST(TGT3::T4)),
            0xEF => Some(Instruction::RST(TGT3::T5)),
            0xF7 => Some(Instruction::RST(TGT3::T6)),
            0xFF => Some(Instruction::RST(TGT3::T7)),

            // DI
            0xF3 => Some(Instruction::DI()),
            // EI
            0xFB => Some(Instruction::EI()),

            // EXPAND to 16-bit OPCODES
            0xCB => Some(Instruction::OPCODE16()),

            // Undocumented OPCODE
            _ => Some(Instruction::NOP()),
        }
    }

    pub fn from_byte_0xcb(byte: u8) -> Option<Instruction> {
        match byte {
            // 0x0x
            0x00 => Some(Instruction::RLC(R8::B)),
            0x01 => Some(Instruction::RLC(R8::C)),
            0x02 => Some(Instruction::RLC(R8::D)),
            0x03 => Some(Instruction::RLC(R8::E)),
            0x04 => Some(Instruction::RLC(R8::H)),
            0x05 => Some(Instruction::RLC(R8::L)),
            0x06 => Some(Instruction::RLC(R8::HL)),
            0x07 => Some(Instruction::RLC(R8::A)),
            0x08 => Some(Instruction::RRC(R8::B)),
            0x09 => Some(Instruction::RRC(R8::C)),
            0x0A => Some(Instruction::RRC(R8::D)),
            0x0B => Some(Instruction::RRC(R8::E)),
            0x0C => Some(Instruction::RRC(R8::H)),
            0x0D => Some(Instruction::RRC(R8::L)),
            0x0E => Some(Instruction::RRC(R8::HL)),
            0x0F => Some(Instruction::RRC(R8::A)),

            // 0x1x
            0x10 => Some(Instruction::RL(R8::B)),
            0x11 => Some(Instruction::RL(R8::C)),
            0x12 => Some(Instruction::RL(R8::D)),
            0x13 => Some(Instruction::RL(R8::E)),
            0x14 => Some(Instruction::RL(R8::H)),
            0x15 => Some(Instruction::RL(R8::L)),
            0x16 => Some(Instruction::RL(R8::HL)),
            0x17 => Some(Instruction::RL(R8::A)),
            0x18 => Some(Instruction::RR(R8::B)),
            0x19 => Some(Instruction::RR(R8::C)),
            0x1A => Some(Instruction::RR(R8::D)),
            0x1B => Some(Instruction::RR(R8::E)),
            0x1C => Some(Instruction::RR(R8::H)),
            0x1D => Some(Instruction::RR(R8::L)),
            0x1E => Some(Instruction::RR(R8::HL)),
            0x1F => Some(Instruction::RR(R8::A)),

            // 0x2x
            0x20 => Some(Instruction::SLA(R8::B)),
            0x21 => Some(Instruction::SLA(R8::C)),
            0x22 => Some(Instruction::SLA(R8::D)),
            0x23 => Some(Instruction::SLA(R8::E)),
            0x24 => Some(Instruction::SLA(R8::H)),
            0x25 => Some(Instruction::SLA(R8::L)),
            0x26 => Some(Instruction::SLA(R8::HL)),
            0x27 => Some(Instruction::SLA(R8::A)),
            0x28 => Some(Instruction::SRA(R8::B)),
            0x29 => Some(Instruction::SRA(R8::C)),
            0x2A => Some(Instruction::SRA(R8::D)),
            0x2B => Some(Instruction::SRA(R8::E)),
            0x2C => Some(Instruction::SRA(R8::H)),
            0x2D => Some(Instruction::SRA(R8::L)),
            0x2E => Some(Instruction::SRA(R8::HL)),
            0x2F => Some(Instruction::SRA(R8::A)),

            // 0x3x
            0x30 => Some(Instruction::SWAP(R8::B)),
            0x31 => Some(Instruction::SWAP(R8::C)),
            0x32 => Some(Instruction::SWAP(R8::D)),
            0x33 => Some(Instruction::SWAP(R8::E)),
            0x34 => Some(Instruction::SWAP(R8::H)),
            0x35 => Some(Instruction::SWAP(R8::L)),
            0x36 => Some(Instruction::SWAP(R8::HL)),
            0x37 => Some(Instruction::SWAP(R8::A)),
            0x38 => Some(Instruction::SRL(R8::B)),
            0x39 => Some(Instruction::SRL(R8::C)),
            0x3A => Some(Instruction::SRL(R8::D)),
            0x3B => Some(Instruction::SRL(R8::E)),
            0x3C => Some(Instruction::SRL(R8::H)),
            0x3D => Some(Instruction::SRL(R8::L)),
            0x3E => Some(Instruction::SRL(R8::HL)),
            0x3F => Some(Instruction::SRL(R8::A)),

            // 0x4x
            0x40 => Some(Instruction::BIT0(R8::B)),
            0x41 => Some(Instruction::BIT0(R8::C)),
            0x42 => Some(Instruction::BIT0(R8::D)),
            0x43 => Some(Instruction::BIT0(R8::E)),
            0x44 => Some(Instruction::BIT0(R8::H)),
            0x45 => Some(Instruction::BIT0(R8::L)),
            0x46 => Some(Instruction::BIT0(R8::HL)),
            0x47 => Some(Instruction::BIT0(R8::A)),
            0x48 => Some(Instruction::BIT1(R8::B)),
            0x49 => Some(Instruction::BIT1(R8::C)),
            0x4A => Some(Instruction::BIT1(R8::D)),
            0x4B => Some(Instruction::BIT1(R8::E)),
            0x4C => Some(Instruction::BIT1(R8::H)),
            0x4D => Some(Instruction::BIT1(R8::L)),
            0x4E => Some(Instruction::BIT1(R8::HL)),
            0x4F => Some(Instruction::BIT1(R8::A)),

            // 0x5x
            0x50 => Some(Instruction::BIT2(R8::B)),
            0x51 => Some(Instruction::BIT2(R8::C)),
            0x52 => Some(Instruction::BIT2(R8::D)),
            0x53 => Some(Instruction::BIT2(R8::E)),
            0x54 => Some(Instruction::BIT2(R8::H)),
            0x55 => Some(Instruction::BIT2(R8::L)),
            0x56 => Some(Instruction::BIT2(R8::HL)),
            0x57 => Some(Instruction::BIT2(R8::A)),
            0x58 => Some(Instruction::BIT3(R8::B)),
            0x59 => Some(Instruction::BIT3(R8::C)),
            0x5A => Some(Instruction::BIT3(R8::D)),
            0x5B => Some(Instruction::BIT3(R8::E)),
            0x5C => Some(Instruction::BIT3(R8::H)),
            0x5D => Some(Instruction::BIT3(R8::L)),
            0x5E => Some(Instruction::BIT3(R8::HL)),
            0x5F => Some(Instruction::BIT3(R8::A)),

            // 0x6x
            0x60 => Some(Instruction::BIT4(R8::B)),
            0x61 => Some(Instruction::BIT4(R8::C)),
            0x62 => Some(Instruction::BIT4(R8::D)),
            0x63 => Some(Instruction::BIT4(R8::E)),
            0x64 => Some(Instruction::BIT4(R8::H)),
            0x65 => Some(Instruction::BIT4(R8::L)),
            0x66 => Some(Instruction::BIT4(R8::HL)),
            0x67 => Some(Instruction::BIT4(R8::A)),
            0x68 => Some(Instruction::BIT5(R8::B)),
            0x69 => Some(Instruction::BIT5(R8::C)),
            0x6A => Some(Instruction::BIT5(R8::D)),
            0x6B => Some(Instruction::BIT5(R8::E)),
            0x6C => Some(Instruction::BIT5(R8::H)),
            0x6D => Some(Instruction::BIT5(R8::L)),
            0x6E => Some(Instruction::BIT5(R8::HL)),
            0x6F => Some(Instruction::BIT5(R8::A)),

            // 0x7x
            0x70 => Some(Instruction::BIT6(R8::B)),
            0x71 => Some(Instruction::BIT6(R8::C)),
            0x72 => Some(Instruction::BIT6(R8::D)),
            0x73 => Some(Instruction::BIT6(R8::E)),
            0x74 => Some(Instruction::BIT6(R8::H)),
            0x75 => Some(Instruction::BIT6(R8::L)),
            0x76 => Some(Instruction::BIT6(R8::HL)),
            0x77 => Some(Instruction::BIT6(R8::A)),
            0x78 => Some(Instruction::BIT7(R8::B)),
            0x79 => Some(Instruction::BIT7(R8::C)),
            0x7A => Some(Instruction::BIT7(R8::D)),
            0x7B => Some(Instruction::BIT7(R8::E)),
            0x7C => Some(Instruction::BIT7(R8::H)),
            0x7D => Some(Instruction::BIT7(R8::L)),
            0x7E => Some(Instruction::BIT7(R8::HL)),
            0x7F => Some(Instruction::BIT7(R8::A)),

            // 0x8x
            0x80 => Some(Instruction::RES0(R8::B)),
            0x81 => Some(Instruction::RES0(R8::C)),
            0x82 => Some(Instruction::RES0(R8::D)),
            0x83 => Some(Instruction::RES0(R8::E)),
            0x84 => Some(Instruction::RES0(R8::H)),
            0x85 => Some(Instruction::RES0(R8::L)),
            0x86 => Some(Instruction::RES0(R8::HL)),
            0x87 => Some(Instruction::RES0(R8::A)),
            0x88 => Some(Instruction::RES1(R8::B)),
            0x89 => Some(Instruction::RES1(R8::C)),
            0x8A => Some(Instruction::RES1(R8::D)),
            0x8B => Some(Instruction::RES1(R8::E)),
            0x8C => Some(Instruction::RES1(R8::H)),
            0x8D => Some(Instruction::RES1(R8::L)),
            0x8E => Some(Instruction::RES1(R8::HL)),
            0x8F => Some(Instruction::RES1(R8::A)),

            // 0x9x
            0x90 => Some(Instruction::RES2(R8::B)),
            0x91 => Some(Instruction::RES2(R8::C)),
            0x92 => Some(Instruction::RES2(R8::D)),
            0x93 => Some(Instruction::RES2(R8::E)),
            0x94 => Some(Instruction::RES2(R8::H)),
            0x95 => Some(Instruction::RES2(R8::L)),
            0x96 => Some(Instruction::RES2(R8::HL)),
            0x97 => Some(Instruction::RES2(R8::A)),
            0x98 => Some(Instruction::RES3(R8::B)),
            0x99 => Some(Instruction::RES3(R8::C)),
            0x9A => Some(Instruction::RES3(R8::D)),
            0x9B => Some(Instruction::RES3(R8::E)),
            0x9C => Some(Instruction::RES3(R8::H)),
            0x9D => Some(Instruction::RES3(R8::L)),
            0x9E => Some(Instruction::RES3(R8::HL)),
            0x9F => Some(Instruction::RES3(R8::A)),

            // 0xAx
            0xA0 => Some(Instruction::RES4(R8::B)),
            0xA1 => Some(Instruction::RES4(R8::C)),
            0xA2 => Some(Instruction::RES4(R8::D)),
            0xA3 => Some(Instruction::RES4(R8::E)),
            0xA4 => Some(Instruction::RES4(R8::H)),
            0xA5 => Some(Instruction::RES4(R8::L)),
            0xA6 => Some(Instruction::RES4(R8::HL)),
            0xA7 => Some(Instruction::RES4(R8::A)),
            0xA8 => Some(Instruction::RES5(R8::B)),
            0xA9 => Some(Instruction::RES5(R8::C)),
            0xAA => Some(Instruction::RES5(R8::D)),
            0xAB => Some(Instruction::RES5(R8::E)),
            0xAC => Some(Instruction::RES5(R8::H)),
            0xAD => Some(Instruction::RES5(R8::L)),
            0xAE => Some(Instruction::RES5(R8::HL)),
            0xAF => Some(Instruction::RES5(R8::A)),

            // 0xBx
            0xB0 => Some(Instruction::RES6(R8::B)),
            0xB1 => Some(Instruction::RES6(R8::C)),
            0xB2 => Some(Instruction::RES6(R8::D)),
            0xB3 => Some(Instruction::RES6(R8::E)),
            0xB4 => Some(Instruction::RES6(R8::H)),
            0xB5 => Some(Instruction::RES6(R8::L)),
            0xB6 => Some(Instruction::RES6(R8::HL)),
            0xB7 => Some(Instruction::RES6(R8::A)),
            0xB8 => Some(Instruction::RES7(R8::B)),
            0xB9 => Some(Instruction::RES7(R8::C)),
            0xBA => Some(Instruction::RES7(R8::D)),
            0xBB => Some(Instruction::RES7(R8::E)),
            0xBC => Some(Instruction::RES7(R8::H)),
            0xBD => Some(Instruction::RES7(R8::L)),
            0xBE => Some(Instruction::RES7(R8::HL)),
            0xBF => Some(Instruction::RES7(R8::A)),

            // 0xCx
            0xC0 => Some(Instruction::SET0(R8::B)),
            0xC1 => Some(Instruction::SET0(R8::C)),
            0xC2 => Some(Instruction::SET0(R8::D)),
            0xC3 => Some(Instruction::SET0(R8::E)),
            0xC4 => Some(Instruction::SET0(R8::H)),
            0xC5 => Some(Instruction::SET0(R8::L)),
            0xC6 => Some(Instruction::SET0(R8::HL)),
            0xC7 => Some(Instruction::SET0(R8::A)),
            0xC8 => Some(Instruction::SET1(R8::B)),
            0xC9 => Some(Instruction::SET1(R8::C)),
            0xCA => Some(Instruction::SET1(R8::D)),
            0xCB => Some(Instruction::SET1(R8::E)),
            0xCC => Some(Instruction::SET1(R8::H)),
            0xCD => Some(Instruction::SET1(R8::L)),
            0xCE => Some(Instruction::SET1(R8::HL)),
            0xCF => Some(Instruction::SET1(R8::A)),

            // 0xDx
            0xD0 => Some(Instruction::SET2(R8::B)),
            0xD1 => Some(Instruction::SET2(R8::C)),
            0xD2 => Some(Instruction::SET2(R8::D)),
            0xD3 => Some(Instruction::SET2(R8::E)),
            0xD4 => Some(Instruction::SET2(R8::H)),
            0xD5 => Some(Instruction::SET2(R8::L)),
            0xD6 => Some(Instruction::SET2(R8::HL)),
            0xD7 => Some(Instruction::SET2(R8::A)),
            0xD8 => Some(Instruction::SET3(R8::B)),
            0xD9 => Some(Instruction::SET3(R8::C)),
            0xDA => Some(Instruction::SET3(R8::D)),
            0xDB => Some(Instruction::SET3(R8::E)),
            0xDC => Some(Instruction::SET3(R8::H)),
            0xDD => Some(Instruction::SET3(R8::L)),
            0xDE => Some(Instruction::SET3(R8::HL)),
            0xDF => Some(Instruction::SET3(R8::A)),

            // 0xEx
            0xE0 => Some(Instruction::SET4(R8::B)),
            0xE1 => Some(Instruction::SET4(R8::C)),
            0xE2 => Some(Instruction::SET4(R8::D)),
            0xE3 => Some(Instruction::SET4(R8::E)),
            0xE4 => Some(Instruction::SET4(R8::H)),
            0xE5 => Some(Instruction::SET4(R8::L)),
            0xE6 => Some(Instruction::SET4(R8::HL)),
            0xE7 => Some(Instruction::SET4(R8::A)),
            0xE8 => Some(Instruction::SET5(R8::B)),
            0xE9 => Some(Instruction::SET5(R8::C)),
            0xEA => Some(Instruction::SET5(R8::D)),
            0xEB => Some(Instruction::SET5(R8::E)),
            0xEC => Some(Instruction::SET5(R8::H)),
            0xED => Some(Instruction::SET5(R8::L)),
            0xEE => Some(Instruction::SET5(R8::HL)),
            0xEF => Some(Instruction::SET5(R8::A)),

            // 0xFx
            0xF0 => Some(Instruction::SET6(R8::B)),
            0xF1 => Some(Instruction::SET6(R8::C)),
            0xF2 => Some(Instruction::SET6(R8::D)),
            0xF3 => Some(Instruction::SET6(R8::E)),
            0xF4 => Some(Instruction::SET6(R8::H)),
            0xF5 => Some(Instruction::SET6(R8::L)),
            0xF6 => Some(Instruction::SET6(R8::HL)),
            0xF7 => Some(Instruction::SET6(R8::A)),
            0xF8 => Some(Instruction::SET7(R8::B)),
            0xF9 => Some(Instruction::SET7(R8::C)),
            0xFA => Some(Instruction::SET7(R8::D)),
            0xFB => Some(Instruction::SET7(R8::E)),
            0xFC => Some(Instruction::SET7(R8::H)),
            0xFD => Some(Instruction::SET7(R8::L)),
            0xFE => Some(Instruction::SET7(R8::HL)),
            0xFF => Some(Instruction::SET7(R8::A)),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::NOP() => write!(f, "{}", op("NOP")),
            Instruction::STOP() => write!(f, "{}", op("STOP")),
            Instruction::HALT() => write!(f, "{}", op("HALT")),
            Instruction::CPL() => write!(f, "{}", op("CPL")),
            Instruction::EI() => write!(f, "{}", op("EI")),
            Instruction::RET(cc) => match cc {
                CC::NONE => write!(f, "{}", op("RET")),
                _ => {
                    let ccf = format!("{:?}", cc);
                    write!(f, "{} {}", op("RET"), arg1(ccf))
                }
            },
            Instruction::CALL(cc) => match cc {
                CC::NONE => write!(f, "{} {}", op("CALL"), arg1("a16")),
                _ => {
                    let ccf = format!("{:?}", cc);
                    write!(f, "{} {}, {}", op("CALL"), arg1(ccf), arg2("a16"))
                }
            },
            Instruction::JPHL() => write!(f, "{} {}", op("JP"), arg1("HL")),
            Instruction::JP(cc) => match cc {
                CC::NONE => write!(f, "{} {}", op("JP"), arg1("a16")),
                _ => {
                    let ccf = format!("{:?}", cc);
                    write!(f, "{} {}, {}", op("JP"), arg1(ccf), arg2("a16"))
                }
            },
            Instruction::JR(cc) => match cc {
                CC::NONE => write!(f, "{} {}", op("JR"), arg1("s8")),
                _ => {
                    let ccf = format!("{:?}", cc);
                    write!(f, "{} {}, {}", op("JR"), arg1(ccf), arg2("s8"))
                }
            },
            Instruction::LD(r8) => match r8 {
                R8::HL => write!(
                    f,
                    "{} {}, {}",
                    op("LD"),
                    arg1(format!("({:?})", r8)),
                    arg2("r8")
                ),
                _ => write!(
                    f,
                    "{} {}, {}",
                    op("LD"),
                    arg1(format!("{:?}", r8)),
                    arg2("r8")
                ),
            },
            Instruction::XOR(r8)
            | Instruction::AND(r8)
            | Instruction::OR(r8)
            | Instruction::CP(r8)
            | Instruction::INC(r8)
            | Instruction::DEC(r8)
            | Instruction::ADC(r8)
            | Instruction::SUB(r8)
            | Instruction::SBC(r8)
            | Instruction::ADD(r8)
            | Instruction::RLC(r8)
            | Instruction::RRC(r8)
            | Instruction::RL(r8)
            | Instruction::RR(r8)
            | Instruction::SLA(r8)
            | Instruction::SRA(r8)
            | Instruction::SWAP(r8)
            | Instruction::SRL(r8)
            | Instruction::BIT0(r8)
            | Instruction::BIT1(r8)
            | Instruction::BIT2(r8)
            | Instruction::BIT3(r8)
            | Instruction::BIT4(r8)
            | Instruction::BIT5(r8)
            | Instruction::BIT6(r8)
            | Instruction::BIT7(r8)
            | Instruction::RES0(r8)
            | Instruction::RES1(r8)
            | Instruction::RES2(r8)
            | Instruction::RES3(r8)
            | Instruction::RES4(r8)
            | Instruction::RES5(r8)
            | Instruction::RES6(r8)
            | Instruction::RES7(r8)
            | Instruction::SET0(r8)
            | Instruction::SET1(r8)
            | Instruction::SET2(r8)
            | Instruction::SET3(r8)
            | Instruction::SET4(r8)
            | Instruction::SET5(r8)
            | Instruction::SET6(r8)
            | Instruction::SET7(r8) => match r8 {
                R8::HL => {
                    let r8f = format!("({:?})", r8);
                    write!(f, "{} {}", op(format!("{:?}", self)), arg1(r8f))
                }
                _ => {
                    let r8f = format!("{:?}", r8);
                    write!(f, "{} {}", op(format!("{:?}", self)), arg1(r8f))
                }
            },
            Instruction::LD16(r16) => {
                let r16f = format!("{:?}", r16);
                write!(f, "{} {}, {}", op("LD"), arg1(r16f), arg2("d16"))
            }
            Instruction::LDfromA(r16ld) => {
                let operand = format!("({:?})", r16ld);
                write!(
                    f,
                    "{} {}, {}",
                    op("LD"),
                    arg1(
                        operand
                            .replace("m", "-")
                            .replace("p", "+")
                            .replace("A", "a")
                    ),
                    arg2("A")
                )
            }
            Instruction::LDtoA(r16ld) => {
                let operand = format!("({:?})", r16ld);
                write!(
                    f,
                    "{} {}, {}",
                    op("LD"),
                    arg1("A"),
                    arg2(
                        operand
                            .replace("m", "-")
                            .replace("p", "+")
                            .replace("A", "a")
                    )
                )
            }
            Instruction::ADDSP() => write!(f, "{} {}, {}", op("ADD"), arg1("SP"), arg2("s8")),
            Instruction::INC16(r16) => write!(f, "{} {:?}", op("INC"), r16),
            Instruction::DEC16(r16) => write!(f, "{} {:?}", op("DEC"), r16),
            Instruction::CPimm() => write!(f, "{} {}", op("CP"), arg1("d8")),

            _ => write!(f, "{:?} (*)", self),
        }
    }
}

/// Operand data of an instruction. This can be a byte or a word.
/// Typically, this is either the next byte/word, or the contents
/// of memory with a given address.
pub enum OperandData {
    Op8(u8),
    Op16(u16),
    None(),
}

/// A representation of an instruction bundled with the data it acts on.
pub struct RunInstr {
    pub instr: Instruction,
    data: OperandData,
}

impl RunInstr {
    pub fn new(opcode: u8, mem: &Memory, reg: &Registers) -> RunInstr {
        let instruction = Instruction::from_byte(opcode);
        let instr = instruction.unwrap();
        let data = match instr {
            Instruction::ADDimm()
            | Instruction::ADCimm()
            | Instruction::SUBimm()
            | Instruction::SBCimm()
            | Instruction::CPimm()
            | Instruction::ORimm()
            | Instruction::XORimm()
            | Instruction::ANDimm()
            | Instruction::LD(_) => OperandData::Op8(mem.read8(reg.pc)),
            Instruction::LD16(_)
            | Instruction::JP(_)
            | Instruction::CALL(_)
            | Instruction::ADD16(_) => OperandData::Op16(mem.read16(reg.pc)),
            Instruction::RET(_) => OperandData::Op16(mem.read16(reg.sp)),
            Instruction::XOR(ref r8)
            | Instruction::AND(ref r8)
            | Instruction::CP(ref r8)
            | Instruction::INC(ref r8)
            | Instruction::DEC(ref r8)
            | Instruction::ADC(ref r8)
            | Instruction::SUB(ref r8)
            | Instruction::SBC(ref r8)
            | Instruction::ADD(ref r8)
            | Instruction::RLC(ref r8)
            | Instruction::RRC(ref r8)
            | Instruction::RL(ref r8)
            | Instruction::RR(ref r8)
            | Instruction::SLA(ref r8)
            | Instruction::SRA(ref r8)
            | Instruction::SWAP(ref r8)
            | Instruction::SRL(ref r8)
            | Instruction::BIT0(ref r8)
            | Instruction::BIT1(ref r8)
            | Instruction::BIT2(ref r8)
            | Instruction::BIT3(ref r8)
            | Instruction::BIT4(ref r8)
            | Instruction::BIT5(ref r8)
            | Instruction::BIT6(ref r8)
            | Instruction::BIT7(ref r8)
            | Instruction::RES0(ref r8)
            | Instruction::RES1(ref r8)
            | Instruction::RES2(ref r8)
            | Instruction::RES3(ref r8)
            | Instruction::RES4(ref r8)
            | Instruction::RES5(ref r8)
            | Instruction::RES6(ref r8)
            | Instruction::RES7(ref r8)
            | Instruction::SET0(ref r8)
            | Instruction::SET1(ref r8)
            | Instruction::SET2(ref r8)
            | Instruction::SET3(ref r8)
            | Instruction::SET4(ref r8)
            | Instruction::SET5(ref r8)
            | Instruction::SET6(ref r8)
            | Instruction::SET7(ref r8) => match r8 {
                R8::HL => OperandData::Op16(reg.get_hl()),
                _ => OperandData::None(),
            },
            Instruction::LDfromA(ref r16ld) => match r16ld {
                R16LD::BC => OperandData::Op16(mem.read16(reg.get_bc())),
                R16LD::DE => OperandData::Op16(mem.read16(reg.get_de())),
                R16LD::HLp => OperandData::Op16(reg.get_hl()),
                R16LD::HLm => OperandData::Op16(reg.get_hl()),
                R16LD::A16 => OperandData::Op16(mem.read16(reg.pc)),
                R16LD::C => OperandData::Op16(0xFF00 | reg.c as u16),
                R16LD::A8 => OperandData::Op16(0xFF00 | (mem.read8(reg.pc) as u16)),
            },
            Instruction::LDtoA(ref r16ld) => match r16ld {
                R16LD::BC => OperandData::Op16(mem.read16(reg.get_bc())),
                R16LD::DE => OperandData::Op16(mem.read16(reg.get_de())),
                R16LD::HLp => OperandData::Op16(mem.read16(reg.get_hl())),
                R16LD::HLm => OperandData::Op16(mem.read16(reg.get_hl())),
                R16LD::A16 => OperandData::Op16(mem.read16(reg.pc)),
                R16LD::C => OperandData::Op16(mem.read16(0xFF00 | reg.c as u16)),
                R16LD::A8 => OperandData::Op16(0xFF00 | mem.read8(reg.pc) as u16),
            },
            Instruction::ADDSP() => OperandData::Op8(mem.read8(reg.pc)),
            Instruction::JR(_) => {
                let j = mem.read8(reg.pc) as i8;
                OperandData::Op16((((reg.pc + 1) as i32) + (j as i32)) as u16)
            }
            _ => OperandData::None(),
        };
        RunInstr { instr, data }
    }

    pub fn instruction_str(&self) -> String {
        format!("{}", self.instr)
    }

    pub fn operand_str(&self) -> String {
        match self.data {
            OperandData::Op8(u8) => {
                format!("{:#04x}", u8)
            }
            OperandData::Op16(u16) => {
                format!("{:#06x}", u16)
            }
            _ => format!("{}", " "),
        }
    }
}

impl fmt::Display for RunInstr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data {
            OperandData::Op8(u8) => {
                write!(f, "{}   {:#04x}", self.instr, u8)
            }
            OperandData::Op16(u16) => {
                write!(f, "{}   {:#06x}", self.instr, u16)
            }
            _ => write!(f, "{}", self.instr),
        }
    }
}

/// Formats the operation name.
fn op<S: AsRef<str>>(name: S) -> String {
    name.as_ref().to_string()
}
/// Formats the first operand.
fn arg1<S: AsRef<str>>(name: S) -> String {
    name.as_ref().to_string()
}
/// Formats the second operand.
fn arg2<S: AsRef<str>>(name: S) -> String {
    name.as_ref().to_string()
}
