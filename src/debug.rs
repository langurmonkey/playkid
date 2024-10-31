use crate::instruction;
use crate::memory;
use crate::registers;

use instruction::Instruction;
use memory::Memory;
use registers::Registers;

use std::io::{stdin, stdout, Read, Write};

pub struct DebugMonitor {
    debug: bool,
    step: bool,
    breakpoint: u16,
}

impl DebugMonitor {
    pub fn new(debug: bool, step: bool) -> Self {
        DebugMonitor {
            debug,
            step,
            breakpoint: 0x2b0,
        }
    }

    /// Performs a cycle.
    pub fn cycle(
        &mut self,
        cycles: u32,
        pc: u16,
        instr: &Instruction,
        opcode: u8,
        mem: &Memory,
        reg: &Registers,
    ) {
        // Debug if needed.
        let stop = pc == self.breakpoint;
        if stop {
            self.debug = true;
        }
        if self.debug || stop {
            if self.step {
                self.debug_step(pc, reg, mem, instr, opcode, cycles);
            } else {
                self.debug(pc, reg, mem, instr, opcode, cycles);
            }
        }
    }

    /// Prints debug information for a given instruction.
    fn debug(
        &self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        instr: &Instruction,
        opcode: u8,
        cycles: u32,
    ) {
        // pc = reg.pc - 1, because we send in the pc before parsing the instruction.
        let next_byte = mem.read8(reg.pc);
        let next_word = mem.read16(reg.pc);
        println!("Instr:     {}", instr);
        println!("Cycle:     {}", cycles);
        println!(
            "Reg:       AF: {:02X} {:02X}
           BC: {:02X} {:02X}
           DE: {:02X} {:02X}
           HL: {:02X} {:02X}",
            reg.a, reg.f, reg.b, reg.c, reg.d, reg.e, reg.h, reg.l
        );
        println!(
            "Flags:     {} {} {} {}",
            if reg.get_z() { "Z" } else { "_" },
            if reg.get_n() { "N" } else { "_" },
            if reg.get_h() { "H" } else { "_" },
            if reg.get_c() { "C" } else { "_" }
        );
        println!("SP:        {:#06X}", reg.sp);
        println!("PC:        {:#06X}", pc);
        println!("Next b/w:  {:#04x} / {:#06x}", next_byte, next_word);
        println!("LCDC:      {:#04X}", mem.ppu.lcdc);
        println!("STAT:      {:#04X}", mem.ppu.stat);
        println!("LX:        {:#04X}", mem.ppu.lx);
        println!("LY:        {:#04X}", mem.ppu.ly);
        println!("LYC:       {:#04X}", mem.ppu.lyc);
        println!("Opcode:    {:#04x}", opcode);
        println!(
            "Joypad:    {} {} {} {} {} {} {} {}",
            if mem.joypad.up { "↑" } else { "_" },
            if mem.joypad.down { "↓" } else { "_" },
            if mem.joypad.left { "←" } else { "_" },
            if mem.joypad.right { "→" } else { "_" },
            if mem.joypad.a { "A" } else { "_" },
            if mem.joypad.b { "B" } else { "_" },
            if mem.joypad.start { "S" } else { "_" },
            if mem.joypad.select { "s" } else { "_" }
        );
        println!();
    }
    /// Prints debug information for a given instruction, and pauses the
    /// execution until user input.
    fn debug_step(
        &self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        instr: &Instruction,
        opcode: u8,
        cycles: u32,
    ) {
        self.debug(pc, reg, mem, instr, opcode, cycles);
        self.pause();
    }

    /// Pauses until the user presses a key.
    fn pause(&self) {
        let mut stdout = stdout();
        stdout.write(b"Press Enter to continue").unwrap();
        stdout.flush().unwrap();
        stdin().read(&mut [0]).unwrap();
    }
}
