use crate::instruction;
use crate::memory;
use crate::registers;

use colored::{ColoredString, Colorize};
use instruction::RunInstr;
use memory::Memory;
use registers::Registers;
use std::io::{stdin, stdout, Read, Write};
use std::process;

pub struct DebugMonitor {
    debug: bool,
    step: bool,
    breakpoints: Vec<u16>,
}

impl DebugMonitor {
    pub fn new(debug: bool, step: bool) -> Self {
        DebugMonitor {
            debug,
            step,
            breakpoints: Vec::new(),
        }
    }

    /// Performs a cycle.
    pub fn cycle(
        &mut self,
        cycles: u32,
        pc: u16,
        run_instr: &RunInstr,
        opcode: u8,
        mem: &Memory,
        reg: &Registers,
    ) {
        // Debug if needed.
        let stop = self.breakpoints.contains(&pc);
        if self.debug || stop {
            if self.step || stop {
                self.debug_step(pc, reg, mem, run_instr, opcode, cycles);
            } else {
                self.debug(pc, reg, mem, run_instr, opcode, cycles);
            }
        }
    }

    /// Prints debug information for a given instruction.
    fn debug(
        &self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        run_instr: &RunInstr,
        opcode: u8,
        cycles: u32,
    ) {
        // pc = reg.pc - 1, because we send in the pc before parsing the instruction.
        let next_byte = mem.read8(reg.pc);
        let next_word = mem.read16(reg.pc);
        println!("Instr:     {}", run_instr);
        println!("Cycle:     {}", cycles);
        println!(
            "Reg:       {}: {:02x} {:02x}
           {}: {:02x} {:02x}
           {}: {:02x} {:02x}
           {}: {:02x} {:02x}",
            "AF".yellow(),
            reg.a,
            reg.f,
            "BC".yellow(),
            reg.b,
            reg.c,
            "DE".yellow(),
            reg.d,
            reg.e,
            "HL".yellow(),
            reg.h,
            reg.l
        );
        println!(
            "Flags:     {} {} {} {}",
            if reg.get_z() {
                "Z".bright_red()
            } else {
                empty()
            },
            if reg.get_n() {
                "N".bright_red()
            } else {
                empty()
            },
            if reg.get_h() {
                "H".bright_red()
            } else {
                empty()
            },
            if reg.get_c() {
                "C".bright_red()
            } else {
                empty()
            }
        );
        println!("SP:        {:#06x}", reg.sp);
        println!("PC:        {:#06x}", pc);
        println!("Next b/w:  {:#04x} / {:#06x}", next_byte, next_word);
        println!("LCDC:      {:#04x}", mem.ppu.lcdc);
        println!("STAT:      {:#04x}", mem.ppu.stat);
        println!("LX:        {:#04x}", mem.ppu.lx);
        println!("LY:        {:#04x}", mem.ppu.ly);
        println!("LYC:       {:#04x}", mem.ppu.lyc);
        println!("Opcode:    {:#04x}", opcode);
        println!(
            "Joypad:    {} {} {} {} {} {} {} {}",
            if mem.joypad.up {
                "↑".green()
            } else {
                empty()
            },
            if mem.joypad.down {
                "↓".green()
            } else {
                empty()
            },
            if mem.joypad.left {
                "←".green()
            } else {
                empty()
            },
            if mem.joypad.right {
                "→".green()
            } else {
                empty()
            },
            if mem.joypad.a { "A".magenta() } else { empty() },
            if mem.joypad.b { "B".magenta() } else { empty() },
            if mem.joypad.start {
                "S".yellow()
            } else {
                empty()
            },
            if mem.joypad.select {
                "s".yellow()
            } else {
                empty()
            }
        );
        println!();
    }
    /// Prints debug information for a given instruction, and pauses the
    /// execution until user input.
    fn debug_step(
        &mut self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        instr: &RunInstr,
        opcode: u8,
        cycles: u32,
    ) {
        self.debug(pc, reg, mem, instr, opcode, cycles);
        self.pause();
    }

    /// Pauses until there is a new command.
    fn pause(&mut self) {
        let mut buf = String::new();
        let mut stdout = stdout();
        println!();
        println!("{}", "===========".bold());
        println!("({})    step", "enter".green());
        println!("({})        continue", "c".green());
        println!(
            "({} {})  add breakpoint to $ADDR",
            "b".green(),
            "$ADDR".blue()
        );
        println!("({})   list breakpoints", "b list".green());
        println!("({})        quit", "q".red());
        stdout.write(b"$ ").unwrap();
        stdout.flush().unwrap();
        match stdin().read_line(&mut buf) {
            Ok(bytes) => {
                if bytes <= 0 {
                    self.pause();
                } else {
                    let first = &buf[0..1];
                    match first {
                        "c" => self.step = false,
                        "b" => {
                            // Breakpoint.
                            let b = buf.strip_suffix("\n").unwrap();
                            let mut spl = b.split(" ");
                            spl.next();
                            let second = spl.next();
                            match second {
                                Some(subcommand) => {
                                    match subcommand {
                                        "list" => {
                                            // List breakpoints.
                                            println!("{}:", "Breakpoints".bold());
                                            for (i, addr) in self.breakpoints.iter().enumerate() {
                                                println!("{}: ${:x}", i, addr);
                                            }
                                        }
                                        _ => {
                                            let value = match subcommand.strip_prefix("$") {
                                                Some(s) => s,
                                                None => subcommand,
                                            };
                                            match u16::from_str_radix(value, 16) {
                                                Ok(addr) => {
                                                    self.breakpoints.push(addr);
                                                    println!("Breakpoint set at: ${:x}", addr);
                                                }
                                                Err(err) => {
                                                    println!(
                                            "Error parsing address (must be a 2-byte hex!) ({:?})",
                                            err.kind()
                                        )
                                                }
                                            };
                                        }
                                    }
                                }
                                None => {
                                    println!("Error parsing breakpoint.");
                                }
                            }
                            self.pause();
                        }
                        "q" => {
                            println!("Bye bye!");
                            process::exit(0);
                        }
                        "\n" => self.step = true,
                        _ => self.pause(),
                    }
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
/// Formats the operand data.
fn empty() -> ColoredString {
    "_".truecolor(110, 110, 110)
}
