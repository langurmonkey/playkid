use crate::instruction;
use crate::memory;
use crate::registers;

use instruction::Instruction;
use memory::Memory;
use registers::Registers;

use std::io::{stdin, stdout, Read, Write};

/// Prints debug information for a given instruction.
pub fn debug(pc: u16, reg: &Registers, mem: &Memory, instr: &Instruction, opcode: u8, cycles: u32) {
    // pc = reg.pc - 1, because we send in the pc before parsing the instruction.
    let next_word = mem.read16(reg.pc);
    println!("Cycle:     {}", cycles);
    println!(
        "Reg:       BC:{:02X} {:02X} 
           DE:{:02X} {:02X} 
           HL:{:02X} {:02X} 
           AF:{:02X} {:02X}",
        reg.b, reg.c, reg.d, reg.e, reg.h, reg.l, reg.a, reg.f
    );
    println!(
        "Flags:     {} {} {} {}",
        if reg.get_z() { "Z" } else { "_" },
        if reg.get_n() { "N" } else { "_" },
        if reg.get_h() { "H" } else { "_" },
        if reg.get_c() { "C" } else { "_" }
    );
    println!("PC:        {:#06X}", pc);
    println!("Next word: {:#06x}", next_word);
    println!("Opcode:    {:#04x}", opcode);
    println!("Instr:     {:?}", instr);
    println!();
}
/// Prints debug information for a given instruction, and pauses the
/// execution until user input.
pub fn debug_step(
    pc: u16,
    reg: &Registers,
    mem: &Memory,
    instr: &Instruction,
    opcode: u8,
    cycles: u32,
) {
    debug(pc, reg, mem, instr, opcode, cycles);
    pause();
}

/// Pauses until the user presses a key.
fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}
