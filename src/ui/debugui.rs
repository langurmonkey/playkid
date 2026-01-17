use crate::instruction::RunInstr;
use crate::memory::Memory;
use crate::registers::Registers;
use crate::ui::{
    label::Label, layout::LayoutGroup, layout::Orientation, uimanager::UIManager, uimanager::Widget,
};
use sdl2::pixels::Color;
use std::cell::RefCell;
use std::rc::Rc;

const DARKGRAY: Color = Color::RGB(30, 30, 30);
const LIGHTBLUE: Color = Color::RGB(130, 130, 255);

pub struct DebugUI<'ttf> {
    /// Main layout group.
    pub main_layout: LayoutGroup<'ttf>,
    pub debug_title: Rc<RefCell<Label>>,
    pub bindings: Rc<RefCell<Label>>,
    pub pc: Rc<RefCell<Label>>,
    pub instr: Rc<RefCell<Label>>,
    pub halt: Rc<RefCell<Label>>,
    pub regs: Rc<RefCell<Label>>,
}

impl<'ttf> DebugUI<'ttf> {
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
        let bindings = Rc::new(RefCell::new(Label::new(
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

        // Halted.
        let halt = Rc::new(RefCell::new(Label::new(
            "HALT",
            22,
            0.0,
            0.0,
            Color::RED,
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

        // Layout.
        let mut root = LayoutGroup::new(Orientation::Vertical, 30.0);

        // PC, Instruction, CPU status.
        let mut pc_row = LayoutGroup::new(Orientation::Horizontal, 50.0);
        pc_row.add(Rc::clone(&pc) as Rc<RefCell<dyn Widget>>);
        pc_row.add(Rc::clone(&instr) as Rc<RefCell<dyn Widget>>);
        pc_row.add(Rc::clone(&halt) as Rc<RefCell<dyn Widget>>);
        let pc_row_rc = Rc::new(RefCell::new(pc_row));

        // Registers.
        let mut reg_row = LayoutGroup::new(Orientation::Horizontal, 45.0);
        reg_row.add(Rc::clone(&regs) as Rc<RefCell<dyn Widget>>);
        let reg_row_rc = Rc::new(RefCell::new(reg_row));

        root.add(Rc::clone(&debug_title) as Rc<RefCell<dyn Widget>>);
        root.add(Rc::clone(&bindings) as Rc<RefCell<dyn Widget>>);
        root.add(pc_row_rc);
        root.add(reg_row_rc);

        // Add all to UI manager.
        ui.add_widget(Rc::clone(&debug_title));
        ui.add_widget(Rc::clone(&bindings));
        ui.add_widget(Rc::clone(&pc));
        ui.add_widget(Rc::clone(&instr));
        ui.add_widget(Rc::clone(&halt));
        ui.add_widget(Rc::clone(&regs));

        Self {
            main_layout: root,
            debug_title,
            bindings,
            pc,
            instr,
            halt,
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
        halted: bool,
    ) {
        // PC.
        self.pc.borrow_mut().set_text(&format!("${:04x}", pc));
        // Instruction.
        self.instr.borrow_mut().set_text(&run_instr.to_string());

        // Halted.
        self.halt.borrow_mut().visible(halted);

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
        self.halt.borrow_mut().visible(visible);
        self.regs.borrow_mut().visible(visible);
    }

    /// The debug widgets float beside the LCD
    /// screen, so we need to update their positions.
    pub fn update_positions(&mut self, ui: &UIManager, dx: f32, dy: f32) {
        // This one call handles every nested child and row!
        self.main_layout.layout(ui, dx + 10.0, dy + 10.0);
    }
}
