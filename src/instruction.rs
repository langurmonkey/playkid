/// Enumerates the r8 registers.
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
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

/// Enumerates the R16 registers to be used in (some) load operations.
pub enum R16LD {
    BC,
    DE,
    HLp,
    HLm,
}

/// Enumerates jump conditions, mostly flags.
pub enum CC {
    NONE,
    NZ,
    Z,
    NC,
    C,
}

/// This enum contains all supported instructions.
/// Instructions that can act with both r8 and r16 registers are first named
/// by default in their r8 version. The r16 versions add a `16` at the end of the name.
pub enum Instruction {
    // NoOp.
    NOP(),
    // Stop.
    STOP(),
    // Halt.
    HALT(),

    // Jump.
    JP(),
    // Relative jumps.
    JR(CC),

    // Load.
    LD(R8),
    LDcp(R8, R8),
    LD16(R16),
    LDfromA(R16LD),
    LDtoA(R16LD),
    LDfromSP(),

    // Increment and decrement.
    INC(R8),
    DEC(R8),
    INC16(R16),
    DEC16(R16),

    // 8-bit arithmetic.
    ADD16(R16),
    ADD(R8),
    ADC(R8),
    SUB(R8),
    SBC(R8),
    AND(R8),
    XOR(R8),
    OR(R8),
    CP(R8),

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

            // LDfromA
            0x02 => Some(Instruction::LDfromA(R16LD::BC)),
            0x12 => Some(Instruction::LDfromA(R16LD::DE)),
            0x22 => Some(Instruction::LDfromA(R16LD::HLp)),
            0x32 => Some(Instruction::LDfromA(R16LD::HLm)),

            // LDtoA
            0x0A => Some(Instruction::LDtoA(R16LD::BC)),
            0x1A => Some(Instruction::LDtoA(R16LD::DE)),
            0x2A => Some(Instruction::LDtoA(R16LD::HLp)),
            0x3A => Some(Instruction::LDtoA(R16LD::HLm)),

            // ADD (r16)
            0x09 => Some(Instruction::ADD16(R16::BC)),
            0x19 => Some(Instruction::ADD16(R16::DE)),
            0x29 => Some(Instruction::ADD16(R16::HL)),
            0x39 => Some(Instruction::ADD16(R16::SP)),
            // ADD (r8)
            0x80 => Some(Instruction::ADD(R8::B)),
            0x81 => Some(Instruction::ADD(R8::C)),
            0x82 => Some(Instruction::ADD(R8::D)),
            0x83 => Some(Instruction::ADD(R8::E)),
            0x84 => Some(Instruction::ADD(R8::H)),
            0x85 => Some(Instruction::ADD(R8::L)),
            0x86 => Some(Instruction::ADD(R8::HL)),
            0x87 => Some(Instruction::ADD(R8::A)),
            // ADC
            0x88 => Some(Instruction::ADC(R8::B)),
            0x89 => Some(Instruction::ADC(R8::C)),
            0x8A => Some(Instruction::ADC(R8::D)),
            0x8B => Some(Instruction::ADC(R8::E)),
            0x8C => Some(Instruction::ADC(R8::H)),
            0x8D => Some(Instruction::ADC(R8::L)),
            0x8E => Some(Instruction::ADC(R8::HL)),
            0x8F => Some(Instruction::ADC(R8::A)),

            // SUB
            0x90 => Some(Instruction::SUB(R8::B)),
            0x91 => Some(Instruction::SUB(R8::C)),
            0x92 => Some(Instruction::SUB(R8::D)),
            0x93 => Some(Instruction::SUB(R8::E)),
            0x94 => Some(Instruction::SUB(R8::H)),
            0x95 => Some(Instruction::SUB(R8::L)),
            0x96 => Some(Instruction::SUB(R8::HL)),
            0x97 => Some(Instruction::SUB(R8::A)),
            // SBC
            0x98 => Some(Instruction::SBC(R8::B)),
            0x99 => Some(Instruction::SBC(R8::C)),
            0x9A => Some(Instruction::SBC(R8::D)),
            0x9B => Some(Instruction::SBC(R8::E)),
            0x9C => Some(Instruction::SBC(R8::H)),
            0x9D => Some(Instruction::SBC(R8::L)),
            0x9E => Some(Instruction::SBC(R8::HL)),
            0x9F => Some(Instruction::SBC(R8::A)),

            // AND
            0xA0 => Some(Instruction::AND(R8::B)),
            0xA1 => Some(Instruction::AND(R8::C)),
            0xA2 => Some(Instruction::AND(R8::D)),
            0xA3 => Some(Instruction::AND(R8::E)),
            0xA4 => Some(Instruction::AND(R8::H)),
            0xA5 => Some(Instruction::AND(R8::L)),
            0xA6 => Some(Instruction::AND(R8::HL)),
            0xA7 => Some(Instruction::AND(R8::A)),
            // XOR
            0xA8 => Some(Instruction::XOR(R8::B)),
            0xA9 => Some(Instruction::XOR(R8::C)),
            0xAA => Some(Instruction::XOR(R8::D)),
            0xAB => Some(Instruction::XOR(R8::E)),
            0xAC => Some(Instruction::XOR(R8::H)),
            0xAD => Some(Instruction::XOR(R8::L)),
            0xAE => Some(Instruction::XOR(R8::HL)),
            0xAF => Some(Instruction::XOR(R8::A)),
            // OR
            0xB0 => Some(Instruction::OR(R8::B)),
            0xB1 => Some(Instruction::OR(R8::C)),
            0xB2 => Some(Instruction::OR(R8::D)),
            0xB3 => Some(Instruction::OR(R8::E)),
            0xB4 => Some(Instruction::OR(R8::H)),
            0xB5 => Some(Instruction::OR(R8::L)),
            0xB6 => Some(Instruction::OR(R8::HL)),
            0xB7 => Some(Instruction::OR(R8::A)),
            // CP
            0xB8 => Some(Instruction::CP(R8::B)),
            0xB9 => Some(Instruction::CP(R8::C)),
            0xBA => Some(Instruction::CP(R8::D)),
            0xBB => Some(Instruction::CP(R8::E)),
            0xBC => Some(Instruction::CP(R8::H)),
            0xBD => Some(Instruction::CP(R8::L)),
            0xBE => Some(Instruction::CP(R8::HL)),
            0xBF => Some(Instruction::CP(R8::A)),

            // JP
            0xE9 => Some(Instruction::JP()),
            // JR
            0x18 => Some(Instruction::JR(CC::NONE)),
            0x20 => Some(Instruction::JR(CC::NZ)),
            0x28 => Some(Instruction::JR(CC::Z)),
            0x30 => Some(Instruction::JR(CC::NC)),
            0x38 => Some(Instruction::JR(CC::C)),

            // INC16
            0x03 => Some(Instruction::INC16(R16::BC)),
            0x13 => Some(Instruction::INC16(R16::DE)),
            0x23 => Some(Instruction::INC16(R16::HL)),
            0x33 => Some(Instruction::INC16(R16::SP)),
            // DEC16
            0x0B => Some(Instruction::DEC16(R16::BC)),
            0x1B => Some(Instruction::DEC16(R16::DE)),
            0x2B => Some(Instruction::DEC16(R16::HL)),
            0x3B => Some(Instruction::DEC16(R16::SP)),
            // INC
            0x04 => Some(Instruction::INC(R8::B)),
            0x0C => Some(Instruction::INC(R8::C)),
            0x14 => Some(Instruction::INC(R8::D)),
            0x1C => Some(Instruction::INC(R8::E)),
            0x24 => Some(Instruction::INC(R8::H)),
            0x2C => Some(Instruction::INC(R8::L)),
            0x34 => Some(Instruction::INC(R8::HL)),
            0x3C => Some(Instruction::INC(R8::A)),
            // DEC
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

            // Not found!
            _ => panic!("Instruction is not implemented: {:#04X}", byte),
        }
    }
}
