use crate::instruction;
use crate::memory;

use instruction::Instruction;
use memory::Memory;

use std::io::{stdin, stdout, Read, Write};

/// Helps with debugging instructions.
pub fn debug(pc: u16, mem: &Memory, instr: &Instruction, opcode: u8) {
    let next_word = mem.read16(pc);
    println!("PC:          {:#x}", pc);
    println!("Next word:   {:#x}", next_word);
    println!("Opcode:      {:#x}", opcode);
    println!("Instruction: {:?}", instr);
    println!();
    pause();
}

/// Pauses until the user presses a key.
fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}
