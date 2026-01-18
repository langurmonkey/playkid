use crate::instruction::RunInstr;
use crate::memory::Memory;
use crate::registers::Registers;
use crate::ui::{
    button::Button, label::Label, layout::LayoutGroup, layout::Orientation, textfield::TextField,
    uimanager::UIManager, uimanager::UIState, uimanager::Widget,
};
use colored::Colorize;
use sdl2::pixels::Color;
use std::cell::RefCell;
use std::rc::Rc;

// Color palette.
const BLUE: Color = Color::RGB(66, 133, 244);
const GRAY: Color = Color::RGB(154, 160, 166);
const WHITE: Color = Color::RGB(255, 255, 255);
const CYAN: Color = Color::RGB(0, 188, 212);
const MAGENTA: Color = Color::RGB(233, 30, 99);
const YELLOW: Color = Color::RGB(244, 180, 0);
const GREEN: Color = Color::RGB(15, 157, 88);
const RED: Color = Color::RGB(219, 68, 55);
const ORANGE: Color = Color::RGB(255, 152, 0);
const DARKGRAY: Color = Color::RGB(30, 30, 30);
const BLACK: Color = Color::RGB(10, 10, 10);

/// The debug user interface.
pub struct DebugUI<'ttf> {
    /// Main layout.
    pub main_layout: Rc<RefCell<LayoutGroup<'ttf>>>,

    /// Status and Instructions
    pub pc_addr: Rc<RefCell<Label>>,
    pub instr_text: Rc<RefCell<Label>>,
    pub instr_operand: Rc<RefCell<Label>>,
    pub status: Rc<RefCell<Label>>,

    /// Values (Right Column)
    pub t_cycles: Rc<RefCell<Label>>,
    pub m_cycles: Rc<RefCell<Label>>,
    pub af: Rc<RefCell<Label>>,
    pub bc: Rc<RefCell<Label>>,
    pub de: Rc<RefCell<Label>>,
    pub hl: Rc<RefCell<Label>>,
    pub flags: Rc<RefCell<Label>>,
    pub sp: Rc<RefCell<Label>>,
    pub div: Rc<RefCell<Label>>,
    pub next_bw: Rc<RefCell<Label>>,
    pub lcdc: Rc<RefCell<Label>>,
    pub stat: Rc<RefCell<Label>>,
    pub lyc: Rc<RefCell<Label>>,
    pub ly: Rc<RefCell<Label>>,
    pub lx: Rc<RefCell<Label>>,
    pub opcode: Rc<RefCell<Label>>,
    pub joypad: Rc<RefCell<Label>>,
}

impl<'ttf> DebugUI<'ttf> {
    pub fn new(ui: &mut UIManager<'ttf>, ui_state: Rc<RefCell<UIState>>) -> Self {
        // Main font size.
        let base_font_size = 10;

        // Title.
        let debug_title = Rc::new(RefCell::new(Label::new(
            "PLAY KID - DEBUG INTERFACE",
            18,
            0.0,
            0.0,
            BLUE,
            None,
            false,
        )));

        // Bindings.
        let mut b_row = LayoutGroup::new(Orientation::Horizontal, 25.0);
        let bindings = [
            ("Step [F6]", YELLOW),
            ("Scanline [F8]", YELLOW),
            ("FPS [F]", YELLOW),
            ("Quit [Esc]", YELLOW),
        ];
        for (txt, clr) in bindings {
            let label = txt.to_string();
            let ui_state_b = Rc::clone(&ui_state);
            b_row.add(Rc::new(RefCell::new(Button::new(
                txt,
                base_font_size,
                clr,
                DARKGRAY, // Normal color.
                RED,      // Pressed color.
                move || {
                    let mut state = ui_state_b.borrow_mut();
                    // Match against the label to decide what to do
                    match label.as_str() {
                        "Step [F6]" => state.step_requested = true,
                        "Scanline [F8]" => state.scanline_requested = true,
                        "FPS [F]" => state.fps_requested = true,
                        "Quit [Esc]" => state.exit_requested = true,
                        _ => println!("{}: Unknown button: {}", "ERR".red(), label),
                    }
                },
            ))));
        }
        let b_row_rc = Rc::new(RefCell::new(b_row));

        // Instruction row.
        let pc_addr = Rc::new(RefCell::new(Label::new(
            "$0000:",
            base_font_size + 4,
            0.0,
            0.0,
            ORANGE,
            None,
            false,
        )));
        let instr_text = Rc::new(RefCell::new(Label::new(
            "NOP",
            base_font_size + 4,
            0.0,
            0.0,
            CYAN,
            None,
            false,
        )));
        let instr_operand = Rc::new(RefCell::new(Label::new(
            "NOP",
            base_font_size + 4,
            0.0,
            0.0,
            GRAY,
            None,
            false,
        )));

        let mut instr_row = LayoutGroup::new(Orientation::Horizontal, 15.0);
        instr_row.add(Rc::clone(&pc_addr) as Rc<RefCell<dyn Widget>>);
        instr_row.add(Rc::clone(&instr_text) as Rc<RefCell<dyn Widget>>);
        instr_row.add(Rc::clone(&instr_operand) as Rc<RefCell<dyn Widget>>);

        // Data table.
        let mut left_col = LayoutGroup::new(Orientation::Vertical, 8.0);
        let labels = [
            "CPU status:",
            "T-cycles:",
            "M-cycles:",
            "Reg:",
            "  ",
            "  ",
            "  ",
            "Flags:",
            "SP:",
            "(i)DIV:",
            "Next b/w:",
            "LCDC:",
            "STAT:",
            "LYC:",
            "LY:",
            "LX:",
            "Opcode:",
            "Joypad:",
        ];
        for text in labels {
            left_col.add(Rc::new(RefCell::new(Label::new(
                text,
                base_font_size,
                0.0,
                0.0,
                GRAY,
                None,
                false,
            ))));
        }

        let mut right_col = LayoutGroup::new(Orientation::Vertical, 8.0);
        let status = Rc::new(RefCell::new(Label::new(
            "RUN",
            base_font_size,
            0.0,
            0.0,
            GREEN,
            None,
            false,
        )));
        let t_cycles = Rc::new(RefCell::new(Label::new(
            "0",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let m_cycles = Rc::new(RefCell::new(Label::new(
            "0",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let af = Rc::new(RefCell::new(Label::new(
            "AF: 00 00",
            base_font_size,
            0.0,
            0.0,
            MAGENTA,
            None,
            false,
        )));
        let bc = Rc::new(RefCell::new(Label::new(
            "BC: 00 00",
            base_font_size,
            0.0,
            0.0,
            MAGENTA,
            None,
            false,
        )));
        let de = Rc::new(RefCell::new(Label::new(
            "DE: 00 00",
            base_font_size,
            0.0,
            0.0,
            MAGENTA,
            None,
            false,
        )));
        let hl = Rc::new(RefCell::new(Label::new(
            "HL: 00 00",
            base_font_size,
            0.0,
            0.0,
            MAGENTA,
            None,
            false,
        )));
        let flags = Rc::new(RefCell::new(Label::new(
            "_ _ _ _",
            base_font_size,
            0.0,
            0.0,
            YELLOW,
            None,
            false,
        )));
        let sp = Rc::new(RefCell::new(Label::new(
            "0x0000",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let div = Rc::new(RefCell::new(Label::new(
            "0x0000/00",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let next_bw = Rc::new(RefCell::new(Label::new(
            "0x00 / 0000",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let lcdc = Rc::new(RefCell::new(Label::new(
            "0x00",
            base_font_size,
            0.0,
            0.0,
            GREEN,
            None,
            false,
        )));
        let stat = Rc::new(RefCell::new(Label::new(
            "0x00",
            base_font_size,
            0.0,
            0.0,
            GREEN,
            None,
            false,
        )));
        let lyc = Rc::new(RefCell::new(Label::new(
            "0x00",
            base_font_size,
            0.0,
            0.0,
            GREEN,
            None,
            false,
        )));
        let ly = Rc::new(RefCell::new(Label::new(
            "0x00",
            base_font_size,
            0.0,
            0.0,
            GREEN,
            None,
            false,
        )));
        let lx = Rc::new(RefCell::new(Label::new(
            "0x00",
            base_font_size,
            0.0,
            0.0,
            GREEN,
            None,
            false,
        )));
        let opcode = Rc::new(RefCell::new(Label::new(
            "0x00",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let joypad = Rc::new(RefCell::new(Label::new(
            "_ _ _ _ _ _ _ _",
            base_font_size,
            0.0,
            0.0,
            ORANGE,
            None,
            false,
        )));

        let val_refs = [
            &status, &t_cycles, &m_cycles, &af, &bc, &de, &hl, &flags, &sp, &div, &next_bw, &lcdc,
            &stat, &lyc, &ly, &lx, &opcode, &joypad,
        ];
        for v in val_refs {
            right_col.add(Rc::clone(v) as Rc<RefCell<dyn Widget>>);
        }

        let mut data_table = LayoutGroup::new(Orientation::Horizontal, 20.0);
        data_table.add(Rc::new(RefCell::new(left_col)) as Rc<RefCell<dyn Widget>>);
        data_table.add(Rc::new(RefCell::new(right_col)) as Rc<RefCell<dyn Widget>>);

        // Reset button.
        let ui_state_reset = Rc::clone(&ui_state);
        let reset_button = Rc::new(RefCell::new(Button::new(
            "Reset CPU",
            base_font_size,
            WHITE,
            DARKGRAY,
            BLUE,
            move || {
                ui_state_reset.borrow_mut().reset_requested = true;
            },
        )));

        // Breakpoints line.
        let mut br_row = LayoutGroup::new(Orientation::Horizontal, 20.0);
        let br_label = Rc::new(RefCell::new(Label::new(
            "Breakpoint: ",
            base_font_size,
            0.0,
            0.0,
            WHITE,
            None,
            false,
        )));
        let br_input = Rc::new(RefCell::new(TextField::new_text(
            base_font_size,
            "$0000".to_string(),
            WHITE,
        )));
        let br_input_b = Rc::clone(&br_input);
        let br_input_for_closure = Rc::clone(&br_input);
        let ui_state_add = Rc::clone(&ui_state);
        let add_button = Rc::new(RefCell::new(Button::new(
            "Add",
            base_font_size,
            WHITE,
            DARKGRAY,
            BLUE,
            move || {
                let is_error = {
                    let text = br_input_b.borrow().get_text();
                    let value = text.strip_prefix("$").unwrap_or(&text);
                    u16::from_str_radix(value, 16).is_err()
                };

                if is_error {
                    br_input_for_closure.borrow_mut().set_color(RED);
                    println!(
                        "{}: Invalid address: {}",
                        "ERR".red(),
                        br_input_b.borrow().get_text()
                    );
                } else {
                    br_input_for_closure.borrow_mut().set_color(WHITE);

                    let text = br_input_b.borrow().get_text();
                    let value = text.strip_prefix("$").unwrap_or(&text);
                    if let Ok(addr) = u16::from_str_radix(value, 16) {
                        let mut uist = ui_state_add.borrow_mut();
                        uist.br_add_requested = true;
                        uist.br_addr = addr;
                    }
                }
            },
        )));

        br_row.add(br_label as Rc<RefCell<dyn Widget>>);
        br_row.add(br_input as Rc<RefCell<dyn Widget>>);
        br_row.add(add_button as Rc<RefCell<dyn Widget>>);

        // Main assembly
        let mut root = LayoutGroup::new(Orientation::Vertical, 30.0);
        root.add(Rc::clone(&debug_title) as Rc<RefCell<dyn Widget>>);
        root.add(b_row_rc as Rc<RefCell<dyn Widget>>);
        root.add(Rc::new(RefCell::new(instr_row)) as Rc<RefCell<dyn Widget>>);
        root.add(Rc::new(RefCell::new(data_table)) as Rc<RefCell<dyn Widget>>);
        root.add(Rc::new(RefCell::new(br_row)) as Rc<RefCell<dyn Widget>>);
        root.add(reset_button as Rc<RefCell<dyn Widget>>);

        let root_rc = Rc::new(RefCell::new(root));
        ui.add_widget(Rc::clone(&root_rc));

        Self {
            main_layout: root_rc,
            pc_addr,
            instr_text,
            instr_operand,
            status,
            t_cycles,
            m_cycles,
            af,
            bc,
            de,
            hl,
            flags,
            sp,
            div,
            next_bw,
            lcdc,
            stat,
            lyc,
            ly,
            lx,
            opcode,
            joypad,
        }
    }

    pub fn machine_state_update(
        &mut self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        run_instr: &RunInstr,
        opcode: u8,
        t_cycles: u32,
        m_cycles: u32,
        halted: bool,
    ) {
        self.pc_addr.borrow_mut().set_text(&format!("${:04x}:", pc));
        self.instr_text
            .borrow_mut()
            .set_text(&run_instr.instruction_str());
        self.instr_operand
            .borrow_mut()
            .set_text(&run_instr.operand_str());

        // Update CPU Status
        if halted {
            self.status.borrow_mut().set_text("HALTED");
            self.status.borrow_mut().set_color(RED);
        } else {
            self.status.borrow_mut().set_text("RUNNING");
            self.status.borrow_mut().set_color(GREEN);
        }

        // Connect T- and M-cycles.
        self.t_cycles
            .borrow_mut()
            .set_text(&format!("{}", t_cycles));
        self.m_cycles
            .borrow_mut()
            .set_text(&format!("{}", m_cycles));

        // Connect registers.
        self.af
            .borrow_mut()
            .set_text(&format!("AF: {:02X} {:02X}", reg.a, reg.f));
        self.bc
            .borrow_mut()
            .set_text(&format!("BC: {:02X} {:02X}", reg.b, reg.c));
        self.de
            .borrow_mut()
            .set_text(&format!("DE: {:02X} {:02X}", reg.d, reg.e));
        self.hl
            .borrow_mut()
            .set_text(&format!("HL: {:02X} {:02X}", reg.h, reg.l));

        // Connect flags.
        let z = if reg.f & 0x80 != 0 { "Z" } else { "_" };
        let n = if reg.f & 0x40 != 0 { "N" } else { "_" };
        let h = if reg.f & 0x20 != 0 { "H" } else { "_" };
        let c = if reg.f & 0x10 != 0 { "C" } else { "_" };
        self.flags
            .borrow_mut()
            .set_text(&format!("{} {} {} {}", z, n, h, c));

        // Connect SP.
        self.sp.borrow_mut().set_text(&format!("0x{:04x}", reg.sp));
        // Connect Opcode.
        self.opcode
            .borrow_mut()
            .set_text(&format!("0x{:02x}", opcode));

        // Connect LCDC.
        self.lcdc
            .borrow_mut()
            .set_text(&format!("{:#04x}", mem.ppu.lcdc));

        // Connect STAT.
        self.stat
            .borrow_mut()
            .set_text(&format!("{:#04x}", mem.ppu.stat));

        // Connect LYC.
        self.lyc
            .borrow_mut()
            .set_text(&format!("{:#04x}", mem.ppu.lyc));

        // Connect LY.
        self.ly
            .borrow_mut()
            .set_text(&format!("{:#04x}", mem.ppu.ly));

        // Connect LX.
        self.lx
            .borrow_mut()
            .set_text(&format!("{:#04x}", mem.ppu.lx));

        // Connect joypad.
        self.joypad.borrow_mut().set_text(&format!(
            "{} {} {} {} {} {} {} {}",
            if mem.joypad.up { "↑" } else { "_" },
            if mem.joypad.down { "↓" } else { "_" },
            if mem.joypad.left { "←" } else { "_" },
            if mem.joypad.right { "→" } else { "_" },
            if mem.joypad.a { "A" } else { "_" },
            if mem.joypad.b { "B" } else { "_" },
            if mem.joypad.start { "S" } else { "_" },
            if mem.joypad.select { "s" } else { "_" }
        ));

        // Connect DIV.
        self.div.borrow_mut().set_text(&format!(
            "{:#06x} / {:#04x}",
            mem.timer.div16(),
            mem.timer.div()
        ));

        // Connect next byte/word.
        let next_byte = mem.read8(reg.pc);
        let next_word = mem.read16(reg.pc);
        self.next_bw
            .borrow_mut()
            .set_text(&format!("{:#04x} / {:#06x}", next_byte, next_word));
    }

    pub fn update_positions(&mut self, ui: &UIManager, dx: f32, dy: f32) {
        self.main_layout
            .borrow_mut()
            .layout(ui, dx + 10.0, dy + 10.0);
    }

    pub fn set_debug_visibility(&mut self, visible: bool) {
        self.main_layout.borrow_mut().set_visible(visible);
    }
}
