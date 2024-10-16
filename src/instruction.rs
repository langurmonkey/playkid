/// This enum contains all supported instructions.
pub enum Instruction {
    NOP,
    ADD(R8),
    ADC(R8),
    SUB(R8),
    SBC(R8),
    AND(R8),
    XOR(R8),
    OR(R8),
    CP(R8),
}

impl Instruction {
    /// Construct an instruction from its byte representation.
    pub fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            // ADD
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

            // Not found!
            _ => panic!("Instruction is not implemented: {:#04X}", byte),
        }
    }
}

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
    AF,
}
