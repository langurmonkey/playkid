use crate::instruction;
use crate::memory;

use instruction::Instruction;
use memory::Memory;

use std::io::{stdin, stdout, Read, Write};

/// Prints debug information for a given instruction.
pub fn debug(pc: u16, mem: &Memory, instr: &Instruction, opcode: u8, cycles: u32) {
    let next_word = mem.read16(pc);
    println!("Cycle:       {}", cycles);
    println!("PC:          {:#06X}", pc);
    println!("Next word:   {:#06x}", next_word);
    println!("Opcode:      {:#04x}", opcode);
    println!("Instruction: {:?}", instr);
    println!();
}
/// Prints debug information for a given instruction, and pauses the
/// execution until user input.
pub fn debug_step(pc: u16, mem: &Memory, instr: &Instruction, opcode: u8, cycles: u32) {
    debug(pc, mem, instr, opcode, cycles);
    pause();
}

/// Pauses until the user presses a key.
fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}
