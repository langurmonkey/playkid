use crate::instruction::RunInstr;
use crate::memory::Memory;
use crate::registers::Registers;
use crate::ui::{label::Label, uimanager::UIManager, uimanager::Widget};
use sdl2::pixels::Color;
use std::cell::RefCell;
use std::rc::Rc;

const DARKGRAY: Color = Color::RGB(30, 30, 30);
const LIGHTBLUE: Color = Color::RGB(130, 130, 255);

pub struct DebugUI {
    pub debug_title: Rc<RefCell<Label>>,
    pub bindings: Rc<RefCell<Label>>,
    pub pc: Rc<RefCell<Label>>,
    pub instr: Rc<RefCell<Label>>,
    pub regs: Rc<RefCell<Label>>,
}

impl DebugUI {
    pub fn new(ui: &mut UIManager) -> Self {
        // Debug title.
        let debug_title = Rc::new(RefCell::new(Label::new(
            "PLAY KID - DEBUG INTERFACE",
            20,
            0.0,
            0.0,
            LIGHTBLUE,
            None,
            false,
        )));

        // Bindings.
        let step_i = Rc::new(RefCell::new(Label::new(
            "Step insr [F6]   Step scanline [F8]   Exit debug [D]   FPS [F]   Quit [ESC]",
            14,
            0.0,
            0.0,
            Color::YELLOW,
            Some(DARKGRAY),
            false,
        )));

        // PC.
        let pc = Rc::new(RefCell::new(Label::new(
            "$0000",
            14,
            0.0,
            0.0,
            Color::GRAY,
            None,
            false,
        )));
        // Instr.
        let instr = Rc::new(RefCell::new(Label::new(
            "NOP",
            14,
            0.0,
            0.0,
            Color::CYAN,
            None,
            false,
        )));

        // Registers.
        let regs = Rc::new(RefCell::new(Label::new(
            "AF: 0000 BC: 0000",
            12,
            0.0,
            0.0,
            Color::MAGENTA,
            None,
            false,
        )));

        // Register widgets with UI manager.
        ui.add_widget(Rc::clone(&debug_title));
        ui.add_widget(Rc::clone(&step_i));
        ui.add_widget(Rc::clone(&pc));
        ui.add_widget(Rc::clone(&instr));
        ui.add_widget(Rc::clone(&regs));

        Self {
            debug_title,
            bindings: step_i,
            pc,
            instr,
            regs,
        }
    }

    /// Syncs the labels with the actual machine state
    pub fn machine_state_update(
        &mut self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        run_instr: &RunInstr,
        opcode: u8,
        cycles: u32,
    ) {
        // PC.
        self.pc.borrow_mut().set_text(&format!("${:04x}", pc));
        // Instruction.
        self.instr.borrow_mut().set_text(&run_instr.to_string());

        // Registers.
        self.regs.borrow_mut().set_text(&format!(
            "AF: {:02X}{:02X} BC: {:02X}{:02X}",
            reg.a, reg.f, reg.b, reg.c,
        ));
    }

    pub fn set_debug_visibility(&mut self, visible: bool) {
        self.debug_title.borrow_mut().visible(visible);
        self.bindings.borrow_mut().visible(visible);
        self.pc.borrow_mut().visible(visible);
        self.instr.borrow_mut().visible(visible);
        self.regs.borrow_mut().visible(visible);
    }

    /// The debug widgets float beside the LCD
    /// screen, so we need to update their positions.
    pub fn update_positions(&mut self, ui: &UIManager, dx: f32, dy: f32) {
        // Update sizes.
        self.check_sizes(ui);

        // Title.
        let mut t = self.debug_title.borrow_mut();
        t.set_pos(dx + 10.0, dy + 10.0);

        // Bindings.
        let mut bindings = self.bindings.borrow_mut();
        bindings.set_pos(dx + 10.0, dy + 40.0);

        // PC, instr.
        self.pc.borrow_mut().set_pos(dx + 10.0, dy + 80.0);
        self.instr.borrow_mut().set_pos(dx + 120.0, dy + 80.0);

        // Regs.
        self.regs.borrow_mut().set_pos(dx + 10.0, dy + 120.0);
    }

    fn check_sizes(&mut self, ui: &UIManager) {
        let mut title = self.debug_title.borrow_mut();
        if !title.has_size() {
            let font = ui.font(title.get_font_size());
            title.update_size(&font);
        }
        let mut bindings = self.bindings.borrow_mut();
        if !bindings.has_size() {
            let font = ui.font(bindings.get_font_size());
            bindings.update_size(&font);
        }
    }
}
