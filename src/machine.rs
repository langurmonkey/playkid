use crate::cartridge;
use crate::constants;
use crate::debugmanager;
use crate::display;
use crate::eventhandler;
use crate::instruction;
use crate::memory;
use crate::registers;

use crate::ui::uimanager::UIState;
use cartridge::Cartridge;
use colored::Colorize;
use debugmanager::DebugManager;
use display::Display;
use eventhandler::EventHandler;
use instruction::{Instruction, RunInstr, CC, R16, R16EXT, R16LD, R8, TGT3};
use memory::Memory;
use registers::Registers;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{EventPump, Sdl};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

/// Spin threshold in milliseconds. Minimum time to sleep until next frame.
const SPIN_THRESHOLD: std::time::Duration = std::time::Duration::from_millis(2);

/// This is our machine, which contains the registers and the memory, and
/// executes the operations.
pub struct Machine<'a, 'b> {
    /// Our registers.
    pub registers: Registers,
    /// The main memory.
    pub memory: Memory<'a>,
    /// The display.
    display: Display<'b>,
    /// The UI state.
    ui_state: Rc<RefCell<UIState>>,
    /// Interrupt master enable flag.
    ime: bool,
    /// EI operation is delayed by one instruction, so we use this counter.
    ei: u8,
    /// DI operation is delayed by one instruction, so we use this counter.
    di: u8,
    /// Flag that holds the running status.
    running: bool,
    /// CPU halted.
    halted: bool,
    /// T-states, basic unit of time, and 1:1 with the clock.
    t_cycles: u32,
    /// M-cycles, base unit for CPU instructions, and 1:4 with the clock.
    m_cycles: u32,
    /// The debug manager.
    debug: DebugManager,
    /// Print FPS every second.
    fps: bool,
    /// The event pump.
    event_pump: EventPump,
}

impl<'a, 'b> Machine<'a, 'b> {
    /// Create a new instance of the Game Boy.
    pub fn new(
        cart: &'a mut Cartridge,
        sdl: &'b Sdl,
        ttf: &'b sdl2::ttf::Sdl2TtfContext,
        scale: u8,
        debug: bool,
        fps: bool,
    ) -> Self {
        // UI state object.
        let ui_state = Rc::new(RefCell::new(UIState::new()));

        Machine {
            registers: Registers::new(),
            memory: Memory::new(cart, sdl),
            display: Display::new(
                "PlayKid emulator",
                sdl,
                ttf,
                scale,
                debug,
                Rc::clone(&ui_state),
            )
            .expect("Error creating display"),
            ui_state: Rc::clone(&ui_state),
            ime: false,
            ei: 0,
            di: 0,
            running: false,
            halted: false,
            t_cycles: 324,
            m_cycles: 0,
            debug: DebugManager::new(debug),
            fps,
            event_pump: sdl.event_pump().unwrap(),
        }
    }

    /// Resets the state of the machine and all its components.
    pub fn reset(&mut self) {
        // reset the internal state
        self.registers.reset();
        self.memory.reset();
        self.ime = false;
        self.ei = 0;
        self.di = 0;
        self.running = true;
        self.halted = false;
        self.t_cycles = 324;
        self.m_cycles = 0;

        // Clear the display
        self.display.clear();
        self.display.present();
    }

    /// Initialize the Game Boy.
    pub fn init(&mut self) {
        self.memory.initialize_hw_registers();
    }

    /// Starts the execution of the machine.
    pub fn start(&mut self) {
        self.running = true;
        self.display.clear();
        self.display.present();

        if self.fps {
            println!("Target FPS: {}", constants::TARGET_FPS);
        }

        // FPS tracking.
        let mut frame_count = 0u32;
        let mut fps_timer = std::time::Instant::now();
        const FPS_LOG_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

        while self.running {
            let frame_start_time = std::time::Instant::now();

            // Handle events.
            self.handle_events();

            // Check UI state.
            self.handle_ui_commands();

            // Stop at breakpoint?
            if self.debug.has_breakpoint(self.registers.pc) && !self.debug.debugging() {
                // If hit, enable debugging.
                self.debug.set_debugging(true);
                self.display.set_debug(self.debug.debugging());
            }

            // Execute cycles for one full frame.
            if self.debug.debugging() {
                if self.debug.take_step_instruction() {
                    // Handle single instruction step.
                    let (t, m, _) = self.machine_cycle();
                    self.m_cycles += m;
                    self.t_cycles += t;
                    self.display.render_lcd(&self.memory);
                } else if self.debug.take_step_line() {
                    // Handle single scanline step.
                    let current_ly = self.memory.ppu().ly;
                    // Loop until LY changes (or wraps around)
                    while self.memory.ppu().ly == current_ly {
                        let (t, m, _) = self.machine_cycle();
                        self.m_cycles += m;
                        self.t_cycles += t;
                        self.display.render_lcd(&self.memory);
                    }
                }
                // Update debug UI.
                let pc = self.registers.pc;
                let opcode = self.memory.read8(pc);
                let run_instr = RunInstr::new(opcode, &self.memory, &self.registers);
                self.display.machine_state_update(
                    pc,
                    &self.registers,
                    &self.memory,
                    &run_instr,
                    &self.debug,
                    opcode,
                    self.t_cycles,
                    self.m_cycles,
                    self.halted,
                );
            } else {
                // Normal full-speed execution.
                let mut cycles_this_frame: usize = 0;
                while cycles_this_frame < constants::CYCLES_PER_FRAME {
                    let (t, m, r) = self.machine_cycle();
                    if !r {
                        // Early exit due to joypad.
                        break;
                    }
                    self.m_cycles += m;
                    self.t_cycles += t;
                    cycles_this_frame += t as usize;

                    self.display.render_lcd(&self.memory);
                }
            }

            self.display.canvas.flush(self.debug.debugging());

            self.display.render_ui();
            self.display.present();

            frame_count += 1;

            // FPS counter.
            self.display.visible_fps(self.fps);
            if self.fps && fps_timer.elapsed() >= FPS_LOG_INTERVAL {
                let actual_fps = frame_count as f64 / fps_timer.elapsed().as_secs_f64();
                self.display.update_fps(actual_fps);
                frame_count = 0;
                fps_timer = std::time::Instant::now();
            }

            // Now, sleep for the remaining time in the frame.
            self.sleep_next_frame(frame_start_time);
        }
    }

    /// Sleeps until next frame, given a start time.
    fn sleep_next_frame(&self, frame_start_time: std::time::Instant) {
        // Now, sleep for the remaining time in the frame.
        let elapsed = frame_start_time.elapsed();
        if elapsed < constants::TARGET_FRAME_DURATION {
            let remaining = constants::TARGET_FRAME_DURATION - elapsed;
            if remaining > SPIN_THRESHOLD {
                thread::sleep(remaining - SPIN_THRESHOLD);
            }
            // Busy spin for remaining small amount.
            while frame_start_time.elapsed() < constants::TARGET_FRAME_DURATION {}
        }
    }

    /// Updates the IME (Interrupt Master Enable) flag.
    /// This is necessary because the effect of the EI and DI instructions
    /// is delayed by one instruction.
    fn ime_update(&mut self) {
        self.di = match self.di {
            2 => 1,
            1 => {
                self.ime = false;
                0
            }
            _ => 0,
        };
        self.ei = match self.ei {
            2 => 1,
            1 => {
                self.ime = true;
                0
            }
            _ => 0,
        };
    }

    /// Interrupt handling. The IF bit corresponding to this interrupt, and the IME flag
    /// are reset by the CUP. IF acknowledges the interrupt, and IME prevents any other
    /// interrupts from being handled until re-enabled (with RETI).
    /// In this case, the corresponding interrupt handler is called by pushing the PC
    /// to the stack, and then setting it to the address of the interrupt handler.
    fn interrupt_handling(&mut self) -> u32 {
        let pending = self.memory.ie & self.memory.iff;

        // Wake from HALT if any interrupt is pending (even if IME is disabled).
        if pending != 0 {
            if self.halted {
                self.halted = false;
            }
        }

        // If no interrupts are pending OR IME is disabled, return
        if !self.ime || pending == 0 {
            return 0;
        }

        // Reset IME.
        self.ime = false;

        // IE and IF have the following format:
        //
        // | 7  6  5 |    4   |    3   |   2   |   1  |    0   |
        // |    1    | Joypad | Serial | Timer |  LCD | VBlank |
        //
        for i in 0..5 {
            let bit = 1 << i;
            if pending & bit != 0 {
                self.memory.iff &= !bit;
                let pc = self.registers.pc;
                self.push_stack(pc);
                self.registers.pc = match bit {
                    0x01 => 0x0040,
                    0x02 => 0x0048,
                    0x04 => 0x0050,
                    0x08 => 0x0058,
                    0x10 => 0x0060,
                    _ => panic!("Unknown interrupt bit {}", bit),
                };
                break;
            }
        }

        5 // 5 M-cycles = 20 T-cycles
    }

    /// Run a machine cycle.
    fn machine_cycle(&mut self) -> (u32, u32, bool) {
        // CPU instruction.
        // One machine cycle (M-cycle) is 4 clock cycles.
        let mut m_cycles = if self.running {
            if self.halted {
                // CPU is halted, don't execute instructions but still consume 1 M-cycle.
                1
            } else {
                // Run next CPU instruction.
                self.cycle() as u32
            }
        } else {
            // NOOP instruction.
            1
        };
        let mut t_cycles = m_cycles * 4;

        // Memory cycle.
        if m_cycles > 0 {
            // Memory cycle.
            let (_, r) = self.memory.cycle(t_cycles);
            if !r {
                // Close down.
                self.running = false;
                return (0, 0, false);
            }
        } else {
            t_cycles = 0;
            m_cycles = 0;
        }

        // Update IME.
        self.ime_update();

        // Handle interrupts if necessary.
        let interrupt_m_cycles = self.interrupt_handling();

        // Return.
        if interrupt_m_cycles > 0 {
            (interrupt_m_cycles * 4, interrupt_m_cycles, true)
        } else {
            (t_cycles, m_cycles, true)
        }
    }

    /// Checks for commands sent from the UI widgets.
    pub fn handle_ui_commands(&mut self) {
        // Step instruction.
        let step_needed = {
            let mut state = self.ui_state.borrow_mut();
            if state.step_requested {
                state.step_requested = false;
                true
            } else {
                false
            }
        };
        if step_needed {
            self.debug.request_step_instruction();
        }
        // Step line.
        let line_needed = {
            let mut state = self.ui_state.borrow_mut();
            if state.scanline_requested {
                state.scanline_requested = false;
                true
            } else {
                false
            }
        };
        if line_needed {
            self.debug.request_step_scanline();
        }
        // Continue.
        let continue_needed = {
            let mut state = self.ui_state.borrow_mut();
            if state.continue_requested {
                state.continue_requested = false;
                true
            } else {
                false
            }
        };
        if continue_needed {
            self.debug.toggle_debugging();
            self.display.set_debug(self.debug.debugging());
        }
        // Add breakpoint.
        let (br_add_needed, br_addr) = {
            let mut state = self.ui_state.borrow_mut();
            (
                if state.br_add_requested {
                    state.br_add_requested = false;
                    true
                } else {
                    false
                },
                state.br_addr,
            )
        };
        if br_add_needed {
            self.debug.add_breakpoint(br_addr);
        }
        // Reset CPU.
        let reset_needed = {
            let mut state = self.ui_state.borrow_mut();
            if state.reset_requested {
                state.reset_requested = false;
                true
            } else {
                false
            }
        };
        if reset_needed {
            self.reset();
        }
        // Quit.
        let exit_needed = {
            let mut state = self.ui_state.borrow_mut();
            if state.exit_requested {
                state.exit_requested = false;
                true
            } else {
                false
            }
        };
        if exit_needed {
            self.running = false;
            println!("{}: Bye bye!", "OK".green());
        }
    }

    /// Polls the events in the queue of the event pump and redirects them to the
    /// interested partners ;).
    fn handle_events(&mut self) {
        // Reset cycles.
        for event in self.event_pump.poll_iter() {
            let mut handled = false;
            // Handle quit events here.
            if !handled {
                handled = match event {
                    // Quit with `Esc` or `CapsLock`.
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::KeyDown {
                        keycode: Some(Keycode::CapsLock),
                        ..
                    } => {
                        self.running = false;
                        println!("{}: Bye bye!", "OK".green());
                        true
                    }
                    // FPS flag (`f` for FPS).
                    Event::KeyDown {
                        keycode: Some(Keycode::F),
                        ..
                    } => {
                        self.fps = !self.fps;
                        true
                    }
                    _ => false,
                }
            }
            // Handle general emulator events.
            if !handled {
                handled = self.memory.joypad.handle_event(&event)
            }
            // Debug monitor.
            if !handled {
                let d = self.debug.debugging();
                handled = self.debug.handle_event(&event);
                if d != self.debug.debugging() {
                    self.display.set_debug(self.debug.debugging());
                }
            }
            // Forward to display/UI.
            if !handled {
                self.display.handle_event(&event);
            }
        }
    }

    /// Main loop of the machine.
    fn cycle(&mut self) -> u8 {
        // Fetch next instruction, and parse it.
        let opcode = self.read8();
        let run_instr = RunInstr::new(opcode, &self.memory, &self.registers);

        // Execute the instruction.
        self.execute(run_instr, opcode)
    }

    /// Execute a single instruction, and returns the number of cycles it takes.
    fn execute(&mut self, run_instr: RunInstr, opcode: u8) -> u8 {
        // Actually execute the instruction.
        match run_instr.instr {
            // NOP: no operation.
            Instruction::NOP() => 1,
            // STOP
            Instruction::STOP() => {
                self.stop();
                1
            }
            // HALT
            Instruction::HALT() => {
                // HALT only halts if IME is enabled.
                if self.ime {
                    self.halt();
                }
                // Otherwise, HALT acts like NOP.
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
                        self.registers.b = self.memory.read8(self.registers.get_hl());
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
                        self.registers.c = self.memory.read8(self.registers.get_hl());
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
                        self.registers.d = self.memory.read8(self.registers.get_hl());
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
                        self.registers.e = self.memory.read8(self.registers.get_hl());
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
                        self.registers.h = self.memory.read8(self.registers.get_hl());
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
                        self.registers.l = self.memory.read8(self.registers.get_hl());
                        2
                    }
                    R8::A => {
                        self.registers.l = self.registers.a;
                        1
                    }
                },
                R8::HL => match r8_1 {
                    R8::B => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.b);
                        2
                    }
                    R8::C => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.c);
                        2
                    }
                    R8::D => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.d);
                        2
                    }
                    R8::E => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.e);
                        2
                    }
                    R8::H => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.h);
                        2
                    }
                    R8::L => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.l);
                        2
                    }
                    R8::HL => 1,
                    R8::A => {
                        self.memory
                            .write8(self.registers.get_hl(), self.registers.a);
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
                        self.registers.a = self.memory.read8(self.registers.get_hl());
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
                    self.memory.write8(self.registers.get_hl(), val);
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
                    self.memory
                        .write8(self.registers.get_bc(), self.registers.a);
                    2
                }
                R16LD::DE => {
                    self.memory
                        .write8(self.registers.get_de(), self.registers.a);
                    2
                }
                R16LD::HLp => {
                    self.memory
                        .write8(self.registers.get_hl_plus(), self.registers.a);
                    2
                }
                R16LD::HLm => {
                    self.memory
                        .write8(self.registers.get_hl_minus(), self.registers.a);
                    2
                }
                R16LD::A8 => {
                    let val = 0xFF00 | (self.read8() as u16);
                    self.memory.write8(val, self.registers.a);
                    3
                }
                R16LD::C => {
                    self.memory
                        .write8(0xFF00 | (self.registers.c as u16), self.registers.a);
                    2
                }
                R16LD::A16 => {
                    let addr = self.read16();
                    self.memory.write8(addr, self.registers.a);
                    4
                }
            },
            // LD A, x
            Instruction::LDtoA(r16ld) => match r16ld {
                R16LD::BC => {
                    self.registers.a = self.memory.read8(self.registers.get_bc());
                    2
                }
                R16LD::DE => {
                    self.registers.a = self.memory.read8(self.registers.get_de());
                    2
                }
                R16LD::HLp => {
                    self.registers.a = self.memory.read8(self.registers.get_hl_plus());
                    2
                }
                R16LD::HLm => {
                    self.registers.a = self.memory.read8(self.registers.get_hl_minus());
                    2
                }
                R16LD::A8 => {
                    let val = 0xFF00 | (self.read8() as u16);
                    self.registers.a = self.memory.read8(val);
                    3
                }
                R16LD::C => {
                    self.registers.a = self.memory.read8(0xFF00 | (self.registers.c as u16));
                    2
                }
                R16LD::A16 => {
                    let val = self.read16();
                    self.registers.a = self.memory.read8(val);
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
                    let val = self.memory.read8(self.registers.get_hl());
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
                    let val = self.memory.read8(self.registers.get_hl());
                    self.add(val, true);
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
                    let val = self.memory.read8(self.registers.get_hl());
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
                    let val = self.memory.read8(self.registers.get_hl());
                    self.sub(val, true);
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
                    let val = self.memory.read8(self.registers.get_hl());
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
                    let val = self.memory.read8(self.registers.get_hl());
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
                    let val = self.memory.read8(self.registers.get_hl());
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
                    let val = self.memory.read8(self.registers.get_hl());
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
                    if !self.registers.get_z() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::Z => {
                    if self.registers.get_z() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::NC => {
                    if !self.registers.get_c() {
                        self.jp();
                        4
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::C => {
                    if self.registers.get_c() {
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
                    if !self.registers.get_z() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
                CC::Z => {
                    if self.registers.get_z() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
                CC::NC => {
                    if !self.registers.get_c() {
                        self.jr();
                        3
                    } else {
                        self.registers.pc += 1;
                        2
                    }
                }
                CC::C => {
                    if self.registers.get_c() {
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
                    let val_inc = self.inc(self.memory.read8(hl));
                    self.memory.write8(hl, val_inc);
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
                    let val_dec = self.dec(self.memory.read8(hl));
                    self.memory.write8(hl, val_dec);
                    3
                }
                R8::A => {
                    self.registers.a = self.dec(self.registers.a);
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
                let c = self.registers.get_c();
                let h = self.registers.get_h();
                let n = self.registers.get_n();

                let mut adjust = if c { 0x60 } else { 0x00 };
                if h {
                    adjust |= 0x06;
                };
                if !n {
                    if a & 0x0F > 0x09 {
                        adjust |= 0x06;
                    };
                    if a > 0x99 {
                        adjust |= 0x60;
                    };
                    a = a.wrapping_add(adjust);
                } else {
                    a = a.wrapping_sub(adjust);
                }

                self.registers.c(adjust >= 0x60);
                self.registers.h(false);
                self.registers.z(a == 0);
                self.registers.a = a;
                1
            }
            // SCF
            Instruction::SCF() => {
                self.registers.c(true);
                self.registers.h(false);
                self.registers.n(false);
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
                self.registers.c(!self.registers.get_c());
                self.registers.h(false);
                self.registers.n(false);
                1
            }

            // RET
            Instruction::RET(cc) => match cc {
                CC::NZ => {
                    if !self.registers.get_z() {
                        self.registers.pc = self.pop_stack();
                        5
                    } else {
                        2
                    }
                }
                CC::NC => {
                    if !self.registers.get_c() {
                        self.registers.pc = self.pop_stack();
                        5
                    } else {
                        2
                    }
                }
                CC::Z => {
                    if self.registers.get_z() {
                        self.registers.pc = self.pop_stack();
                        5
                    } else {
                        2
                    }
                }
                CC::C => {
                    if self.registers.get_c() {
                        self.registers.pc = self.pop_stack();
                        5
                    } else {
                        2
                    }
                }
                CC::NONE => {
                    self.registers.pc = self.pop_stack();
                    4
                }
            },
            // RETI
            Instruction::RETI() => {
                let val = self.memory.read16(self.registers.sp);
                self.registers.sp += 2;
                self.registers.pc = val;
                self.ei = 1;
                4
            }

            // POP
            Instruction::POP(r16ext) => match r16ext {
                R16EXT::BC => {
                    let value = self.pop_stack();
                    self.registers.set_bc(value);
                    3
                }
                R16EXT::DE => {
                    let value = self.pop_stack();
                    self.registers.set_de(value);
                    3
                }
                R16EXT::HL => {
                    let value = self.pop_stack();
                    self.registers.set_hl(value);
                    3
                }
                R16EXT::AF => {
                    let value = self.pop_stack();
                    self.registers.set_af(value);
                    3
                }
            },

            // PUSH
            Instruction::PUSH(r16ext) => match r16ext {
                R16EXT::BC => {
                    self.push_stack(self.registers.get_bc());
                    4
                }
                R16EXT::DE => {
                    self.push_stack(self.registers.get_de());
                    4
                }
                R16EXT::HL => {
                    self.push_stack(self.registers.get_hl());
                    4
                }
                R16EXT::AF => {
                    self.push_stack(self.registers.get_af());
                    4
                }
            },

            // CALL
            Instruction::CALL(cc) => match cc {
                CC::NZ => {
                    if !self.registers.get_z() {
                        self.push_stack(self.registers.pc + 2);
                        self.registers.pc = self.read16();
                        6
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::NC => {
                    if !self.registers.get_c() {
                        self.push_stack(self.registers.pc + 2);
                        self.registers.pc = self.read16();
                        6
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::Z => {
                    if self.registers.get_z() {
                        self.push_stack(self.registers.pc + 2);
                        self.registers.pc = self.read16();
                        6
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::C => {
                    if self.registers.get_c() {
                        self.push_stack(self.registers.pc + 2);
                        self.registers.pc = self.read16();
                        6
                    } else {
                        self.registers.pc += 2;
                        3
                    }
                }
                CC::NONE => {
                    self.push_stack(self.registers.pc + 2);
                    self.registers.pc = self.read16();
                    6
                }
            },

            // RST
            Instruction::RST(tgt3) => match tgt3 {
                TGT3::T0 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x00;
                    4
                }
                TGT3::T1 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x08;
                    4
                }
                TGT3::T2 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x10;
                    4
                }
                TGT3::T3 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x18;
                    4
                }
                TGT3::T4 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x20;
                    4
                }
                TGT3::T5 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x28;
                    4
                }
                TGT3::T6 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x30;
                    4
                }
                TGT3::T7 => {
                    self.push_stack(self.registers.pc);
                    self.registers.pc = 0x38;
                    4
                }
            },

            // DI
            Instruction::DI() => {
                self.di = 2;
                1
            }
            // EI
            Instruction::EI() => {
                self.ei = 2;
                1
            }
            // OPCODE 16-bit (0xCB).
            Instruction::OPCODE16() => {
                // Read next byte, construct instruction, execute 0xCB instruction.
                let opcode0xcb = self.read8();
                let instr0xcb = Instruction::from_byte_0xcb(opcode0xcb);
                let msg = format!("Incorrect 0xCB opcode: {:#04X}", opcode0xcb);
                self.execute_0xcb(instr0xcb.expect(&msg), opcode0xcb)
            }

            // Never should happen.
            _ => panic!("Instruction is not implemented: {:#04X}", opcode),
        }
    }

    fn execute_0xcb(&mut self, instr: Instruction, opcode: u8) -> u8 {
        match instr {
            Instruction::RLC(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.rlc(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.rlc(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.rlc(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.rlc(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.rlc(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.rlc(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.rlc(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.rlc(self.registers.a);
                    2
                }
            },
            Instruction::RRC(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.rrc(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.rrc(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.rrc(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.rrc(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.rrc(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.rrc(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.rrc(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.rrc(self.registers.a);
                    2
                }
            },
            Instruction::RL(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.rl(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.rl(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.rl(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.rl(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.rl(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.rl(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.rl(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.rl(self.registers.a);
                    2
                }
            },
            Instruction::RR(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.rr(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.rr(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.rr(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.rr(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.rr(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.rr(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.rr(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.rr(self.registers.a);
                    2
                }
            },
            Instruction::SLA(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.sla(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.sla(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.sla(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.sla(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.sla(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.sla(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.sla(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.sla(self.registers.a);
                    2
                }
            },
            Instruction::SRA(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.sra(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.sra(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.sra(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.sra(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.sra(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.sra(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.sra(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.sra(self.registers.a);
                    2
                }
            },
            Instruction::SWAP(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.swap(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.swap(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.swap(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.swap(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.swap(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.swap(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.swap(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.swap(self.registers.a);
                    2
                }
            },
            Instruction::SRL(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.srl(self.registers.b);
                    2
                }
                R8::C => {
                    self.registers.c = self.srl(self.registers.c);
                    2
                }
                R8::D => {
                    self.registers.d = self.srl(self.registers.d);
                    2
                }
                R8::E => {
                    self.registers.e = self.srl(self.registers.e);
                    2
                }
                R8::H => {
                    self.registers.h = self.srl(self.registers.h);
                    2
                }
                R8::L => {
                    self.registers.l = self.srl(self.registers.l);
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let value = self.memory.read8(hl);
                    let value2 = self.srl(value);
                    self.memory.write8(hl, value2);
                    4
                }
                R8::A => {
                    self.registers.a = self.srl(self.registers.a);
                    2
                }
            },
            Instruction::BIT0(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 0);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 0);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 0);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 0);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 0);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 0);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 0);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 0);
                    2
                }
            },
            Instruction::BIT1(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 1);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 1);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 1);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 1);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 1);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 1);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 1);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 1);
                    2
                }
            },
            Instruction::BIT2(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 2);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 2);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 2);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 2);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 2);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 2);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 2);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 2);
                    2
                }
            },
            Instruction::BIT3(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 3);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 3);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 3);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 3);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 3);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 3);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 3);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 3);
                    2
                }
            },
            Instruction::BIT4(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 4);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 4);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 4);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 4);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 4);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 4);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 4);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 4);
                    2
                }
            },
            Instruction::BIT5(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 5);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 5);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 5);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 5);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 5);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 5);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 5);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 5);
                    2
                }
            },
            Instruction::BIT6(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 6);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 6);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 6);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 6);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 6);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 6);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 6);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 6);
                    2
                }
            },
            Instruction::BIT7(r8) => match r8 {
                R8::B => {
                    self.bit(self.registers.b, 7);
                    2
                }
                R8::C => {
                    self.bit(self.registers.c, 7);
                    2
                }
                R8::D => {
                    self.bit(self.registers.d, 7);
                    2
                }
                R8::E => {
                    self.bit(self.registers.e, 7);
                    2
                }
                R8::H => {
                    self.bit(self.registers.h, 7);
                    2
                }
                R8::L => {
                    self.bit(self.registers.l, 7);
                    2
                }
                R8::HL => {
                    let value = self.memory.read8(self.registers.get_hl());
                    self.bit(value, 7);
                    3
                }
                R8::A => {
                    self.bit(self.registers.a, 7);
                    2
                }
            },
            Instruction::RES0(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xFE;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xFE;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xFE;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xFE;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xFE;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xFE;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xFE;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xFE;
                    2
                }
            },
            Instruction::RES1(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xFD;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xFD;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xFD;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xFD;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xFD;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xFD;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xFD;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xFD;
                    2
                }
            },
            Instruction::RES2(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xFB;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xFB;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xFB;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xFB;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xFB;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xFB;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xFB;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xFB;
                    2
                }
            },
            Instruction::RES3(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xF7;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xF7;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xF7;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xF7;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xF7;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xF7;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xF7;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xF7;
                    2
                }
            },
            Instruction::RES4(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xEF;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xEF;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xEF;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xEF;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xEF;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xEF;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xEF;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xEF;
                    2
                }
            },
            Instruction::RES5(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xDF;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xDF;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xDF;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xDF;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xDF;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xDF;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xDF;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xDF;
                    2
                }
            },
            Instruction::RES6(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0xBF;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0xBF;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0xBF;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0xBF;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0xBF;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0xBF;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0xBF;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0xBF;
                    2
                }
            },
            Instruction::RES7(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b & 0x7F;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c & 0x7F;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d & 0x7F;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e & 0x7F;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h & 0x7F;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l & 0x7F;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) & 0x7F;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a & 0x7F;
                    2
                }
            },
            Instruction::SET0(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x01;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x01;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x01;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x01;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x01;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x01;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x01;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x01;
                    2
                }
            },
            Instruction::SET1(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x02;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x02;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x02;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x02;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x02;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x02;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x02;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x02;
                    2
                }
            },
            Instruction::SET2(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x04;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x04;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x04;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x04;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x04;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x04;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x04;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x04;
                    2
                }
            },
            Instruction::SET3(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x08;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x08;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x08;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x08;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x08;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x08;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x08;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x08;
                    2
                }
            },
            Instruction::SET4(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x10;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x10;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x10;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x10;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x10;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x10;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x10;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x10;
                    2
                }
            },
            Instruction::SET5(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x20;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x20;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x20;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x20;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x20;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x20;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x20;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x20;
                    2
                }
            },
            Instruction::SET6(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x40;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x40;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x40;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x40;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x40;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x40;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x40;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x40;
                    2
                }
            },
            Instruction::SET7(r8) => match r8 {
                R8::B => {
                    self.registers.b = self.registers.b | 0x80;
                    2
                }
                R8::C => {
                    self.registers.c = self.registers.c | 0x80;
                    2
                }
                R8::D => {
                    self.registers.d = self.registers.d | 0x80;
                    2
                }
                R8::E => {
                    self.registers.e = self.registers.e | 0x80;
                    2
                }
                R8::H => {
                    self.registers.h = self.registers.h | 0x80;
                    2
                }
                R8::L => {
                    self.registers.l = self.registers.l | 0x80;
                    2
                }
                R8::HL => {
                    let hl = self.registers.get_hl();
                    let val = self.memory.read8(hl) | 0x80;
                    self.memory.write8(hl, val);
                    4
                }
                R8::A => {
                    self.registers.a = self.registers.a | 0x80;
                    2
                }
            },
            // Never should happen.
            _ => panic!("0xCB instruction is not implemented: {:#04X}", opcode),
        }
    }

    /// Halt the machine by setting the `halted` flag to true.
    fn halt(&mut self) {
        self.halted = true;
    }

    /// Stop the machine by setting the `running` flag to false.
    fn stop(&mut self) {
        // Reset DIV register.
        self.memory.write8(0xFF04, 0x00);
        self.running = false;
    }

    /// Reads the next byte in memory at the location of `pc`, and
    /// increments `pc`.
    fn read8(&mut self) -> u8 {
        let result = self.memory.read8(self.registers.pc);
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

    /// Push a word to the stack.
    fn push_stack(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.memory.write16(self.registers.sp, value);
    }

    /// Pop a word from the stack.
    fn pop_stack(&mut self) -> u16 {
        let result = self.memory.read16(self.registers.sp);
        self.registers.sp += 2;
        result
    }

    /// Set `pc` to next 2 bytes in RAM.
    fn jp(&mut self) {
        self.registers.pc = self.read16();
    }

    /// Jump operation.
    fn jr(&mut self) {
        let j = self.read8() as i8;
        self.registers.pc = ((self.registers.pc as i32) + (j as i32)) as u16;
    }

    /// Increment the value, update flags, return.
    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.registers.z(result == 0);
        self.registers.h((value & 0x0F) + 1 > 0x0F);
        self.registers.n(false);
        result
    }

    /// Decrement the value, update flags, return it.
    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.registers.z(result == 0);
        self.registers.h((value & 0x0F) == 0);
        self.registers.n(true);
        result
    }

    /// Set flags after a bit shift operation.
    fn rotate_flags(&mut self, result: u8, carry: bool) {
        self.registers.z(result == 0);
        self.registers.c(carry);
        self.registers.h(false);
        self.registers.n(false);
    }

    /// RLC operation: rotate contents of `val` to the right. Update flags.
    /// The contents of bit 7 are placed in `c` and in bit 0 of `val`.
    fn rlc(&mut self, val: u8) -> u8 {
        let carry = val & 0x80 > 0;
        let result = (val << 1) | (if carry { 1 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    /// RL operation: rotate contents of `val` to the left. Update flags.
    /// The previous contents of the carry `c` flag are copied to bit 0 of `val`.
    fn rl(&mut self, val: u8) -> u8 {
        let carry = val & 0x80 > 0;
        let result = (val << 1) | (if self.registers.get_c() { 1 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    /// RRC operation: rotate contents of `val` to the right. Update flags.
    /// The contents of bit 0 are placed in `c` and in bit 7 of `val`.
    fn rrc(&mut self, val: u8) -> u8 {
        let carry = val & 0x01 > 0;
        let result = (val >> 1) | (if carry { 0x80 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    /// RR operation: rotate contents of `val` to the right. Update flags.
    /// The previous contents of the carry `c` flag are copied to bit 7 of `val`.
    fn rr(&mut self, val: u8) -> u8 {
        let carry = val & 0x01 > 0;
        let result = (val >> 1) | (if self.registers.get_c() { 0x80 } else { 0 });
        self.rotate_flags(result, carry);
        result
    }

    /// SLA operation: shift contents of `val` to the left,
    /// update flags, reset bit 0 of `val` to 0.
    fn sla(&mut self, val: u8) -> u8 {
        let carry = val & 0x80 == 0x80;
        let result = val << 1;
        self.rotate_flags(result, carry);
        result
    }

    /// SRA operation: shift contents of `val` to the
    /// right, update flags, but do not change bit 7 of `val`.
    fn sra(&mut self, val: u8) -> u8 {
        let carry = val & 0x01 == 0x01;
        let result = (val >> 1) | (val & 0x80);
        self.rotate_flags(result, carry);
        result
    }

    /// SWAP operation: shift the contents of the lower-order 4 bits of `val` to the
    /// higher-order 4 bits, and shift the higher-order 4 bits to the lower-order
    /// 4 bits.
    fn swap(&mut self, val: u8) -> u8 {
        self.registers.z(val == 0);
        self.registers.c(false);
        self.registers.h(false);
        self.registers.n(false);
        (val >> 4) | (val << 4)
    }

    /// SRL operation: shift the contents of `val` to the
    /// right, update flags, reset bit 7 of `val` to 0.
    fn srl(&mut self, val: u8) -> u8 {
        let carry = val & 0x01 == 0x01;
        let result = val >> 1;
        self.rotate_flags(result, carry);
        result
    }

    /// BIT operation: copy the complement of the contents of the bit `bit` of `val`
    /// to the `z` flagof the program status word (PSW).
    fn bit(&mut self, val: u8, bit: u8) {
        let result = val & (1 << (bit as u32)) == 0;
        self.registers.n(false);
        self.registers.h(true);
        self.registers.z(result);
    }

    /// Adds the given 16-bit value to `hl`.
    fn add16(&mut self, value: u16) {
        let hl = self.registers.get_hl();
        // Actual addition.
        let result = hl.wrapping_add(value);
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
        let carry = if use_carry && self.registers.get_c() {
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
        // lower nibbles of `a` and target overflows.
        self.registers.h((a & 0x0F) + (value & 0x0F) + carry > 0x0F);

        // Result -> a.
        self.registers.a = result;
    }

    fn sub(&mut self, value: u8, use_carry: bool) {
        // Get carry if needed.
        let carry = if use_carry && self.registers.get_c() {
            1
        } else {
            0
        };
        let a = self.registers.a;
        // Actual subtraction.
        let result = a.wrapping_sub(value).wrapping_sub(carry);
        // Update zero flag.
        self.registers.z(result == 0);
        // Update subtraction flag.
        self.registers.n(true);
        // Update carry flag if borrow (value+carry > a).
        self.registers
            .c((value as u16) + (carry as u16) > (a as u16));
        // Update half-carry flag. Set if borrow from bit 4.
        self.registers.h((a & 0x0F) < (value & 0x0F) + carry);

        // Result -> a.
        self.registers.a = result;
    }

    /// Logic AND.
    fn and(&mut self, value: u8) {
        let result = self.registers.a & value;
        self.registers.z(result == 0);
        self.registers.n(false);
        self.registers.c(false);
        self.registers.h(true);
        // Result -> a.
        self.registers.a = result;
    }

    /// Logic XOR.
    fn xor(&mut self, value: u8) {
        let result = self.registers.a ^ value;
        self.registers.z(result == 0);
        self.registers.n(false);
        self.registers.c(false);
        self.registers.h(false);
        // Result -> a.
        self.registers.a = result;
    }

    /// Logic OR.
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
