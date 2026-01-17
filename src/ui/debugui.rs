use crate::instruction::RunInstr;
use crate::memory::Memory;
use crate::registers::Registers;
use crate::ui::{
    label::Label, layout::LayoutGroup, layout::Orientation, uimanager::UIManager, uimanager::Widget,
};
use sdl2::pixels::Color;
use std::cell::RefCell;
use std::rc::Rc;

const BLUE: Color = Color::RGB(66, 133, 244);
const LIGHTBLUE: Color = Color::RGB(99, 199, 255);
const RED: Color = Color::RGB(219, 68, 55);
const YELLOW: Color = Color::RGB(244, 180, 0);
const GREEN: Color = Color::RGB(15, 157, 88);
const CYAN: Color = Color::RGB(0, 188, 212);
const MAGENTA: Color = Color::RGB(233, 30, 99);
const PURPLE: Color = Color::RGB(156, 39, 176);
const ORANGE: Color = Color::RGB(255, 152, 0);
const TEAL: Color = Color::RGB(0, 150, 136);
const GRAY: Color = Color::RGB(90, 90, 90);
const DARKGRAY: Color = Color::RGB(30, 30, 30);

pub struct DebugUI<'ttf> {
    /// Main layout group.
    pub main_layout: Rc<RefCell<LayoutGroup<'ttf>>>,
    pub debug_title: Rc<RefCell<Label>>,
    pub b1: Rc<RefCell<Label>>,
    pub b2: Rc<RefCell<Label>>,
    pub b3: Rc<RefCell<Label>>,
    pub b4: Rc<RefCell<Label>>,
    pub b5: Rc<RefCell<Label>>,
    pub pc: Rc<RefCell<Label>>,
    pub instr: Rc<RefCell<Label>>,
    pub status: Rc<RefCell<Label>>,
    pub cycles: Rc<RefCell<Label>>,
    pub regs: Rc<RefCell<Label>>,
    pub regs2: Rc<RefCell<Label>>,
}

impl<'ttf> DebugUI<'ttf> {
    pub fn new(ui: &mut UIManager<'ttf>) -> Self {
        // Debug title.
        let debug_title = Rc::new(RefCell::new(Label::new(
            "PLAY KID - DEBUG INTERFACE",
            20,
            0.0,
            0.0,
            BLUE,
            None,
            false,
        )));

        // Bindings.
        let b1 = Rc::new(RefCell::new(Label::new(
            "Step insr [F6]",
            10,
            0.0,
            0.0,
            YELLOW,
            Some(DARKGRAY),
            false,
        )));
        let b2 = Rc::new(RefCell::new(Label::new(
            "Step scanline [F8]",
            10,
            0.0,
            0.0,
            YELLOW,
            Some(DARKGRAY),
            false,
        )));
        let b3 = Rc::new(RefCell::new(Label::new(
            "FPS [F]",
            10,
            0.0,
            0.0,
            YELLOW,
            Some(DARKGRAY),
            false,
        )));
        let b4 = Rc::new(RefCell::new(Label::new(
            "Exit debug [D]",
            10,
            0.0,
            0.0,
            YELLOW,
            Some(DARKGRAY),
            false,
        )));
        let b5 = Rc::new(RefCell::new(Label::new(
            "Quit [ESC]",
            10,
            0.0,
            0.0,
            YELLOW,
            Some(DARKGRAY),
            false,
        )));

        // PC.
        let pc = Rc::new(RefCell::new(Label::new(
            "$0000", 10, 0.0, 0.0, GRAY, None, false,
        )));
        // Instr.
        let instr = Rc::new(RefCell::new(Label::new(
            "NOP", 10, 0.0, 0.0, CYAN, None, false,
        )));

        // CPU status.
        let mut s_row = LayoutGroup::new(Orientation::Horizontal, 5.0);
        let cpu_status = Rc::new(RefCell::new(Label::new(
            "CPU status:",
            10,
            0.0,
            0.0,
            GRAY,
            None,
            false,
        )));
        let status = Rc::new(RefCell::new(Label::new(
            "RUN", 10, 0.0, 0.0, GREEN, None, false,
        )));
        s_row.add(Rc::clone(&cpu_status) as Rc<RefCell<dyn Widget>>);
        s_row.add(Rc::clone(&status) as Rc<RefCell<dyn Widget>>);
        let s_row_rc = Rc::new(RefCell::new(s_row));

        // Cycles Label (T and M)
        let cycles = Rc::new(RefCell::new(Label::new(
            "T-cycles: 0   M-cycles: 0",
            10,
            0.0,
            0.0,
            ORANGE,
            None,
            false,
        )));

        // Registers.
        let regs = Rc::new(RefCell::new(Label::new(
            "AF: 0x0000 BC: 0x0000",
            10,
            0.0,
            0.0,
            MAGENTA,
            None,
            false,
        )));
        let regs2 = Rc::new(RefCell::new(Label::new(
            "DE: 0x0000 HL: 0x0000 SP: 0x0000",
            10,
            0.0,
            0.0,
            MAGENTA,
            None,
            false,
        )));

        // Layout.
        let mut root = LayoutGroup::new(Orientation::Vertical, 20.0);
        // Root position is 160 * 1.5 + 10.
        root.set_pos(250.0, 10.0);

        // Bindings.
        let mut b_row = LayoutGroup::new(Orientation::Horizontal, 30.0);
        b_row.add(Rc::clone(&b1) as Rc<RefCell<dyn Widget>>);
        b_row.add(Rc::clone(&b2) as Rc<RefCell<dyn Widget>>);
        b_row.add(Rc::clone(&b3) as Rc<RefCell<dyn Widget>>);
        b_row.add(Rc::clone(&b4) as Rc<RefCell<dyn Widget>>);
        b_row.add(Rc::clone(&b5) as Rc<RefCell<dyn Widget>>);
        let b_row_rc = Rc::new(RefCell::new(b_row));

        // PC, Instruction, CPU status.
        let mut pc_row = LayoutGroup::new(Orientation::Horizontal, 50.0);
        pc_row.add(Rc::clone(&pc) as Rc<RefCell<dyn Widget>>);
        pc_row.add(Rc::clone(&instr) as Rc<RefCell<dyn Widget>>);
        pc_row.add(s_row_rc);
        let pc_row_rc = Rc::new(RefCell::new(pc_row));

        // Cycles and registers.
        let mut reg_stack = LayoutGroup::new(Orientation::Vertical, 8.0);
        reg_stack.add(Rc::clone(&cycles) as Rc<RefCell<dyn Widget>>);
        reg_stack.add(Rc::clone(&regs) as Rc<RefCell<dyn Widget>>);
        reg_stack.add(Rc::clone(&regs2) as Rc<RefCell<dyn Widget>>);
        let reg_stack_rc = Rc::new(RefCell::new(reg_stack));

        root.add(Rc::clone(&debug_title) as Rc<RefCell<dyn Widget>>);
        root.add(Rc::clone(&b_row_rc) as Rc<RefCell<dyn Widget>>);
        root.add(pc_row_rc);
        root.add(reg_stack_rc);

        // Add root to UI manager.
        let root_rc = Rc::new(RefCell::new(root));
        ui.add_widget(Rc::clone(&root_rc));

        Self {
            main_layout: root_rc,
            debug_title,
            b1,
            b2,
            b3,
            b4,
            b5,
            pc,
            instr,
            cycles,
            status,
            regs,
            regs2,
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
        t_cycles: u32,
        m_cycles: u32,
        halted: bool,
    ) {
        // PC.
        self.pc.borrow_mut().set_text(&format!("${:04x}", pc));
        // Instruction.
        self.instr.borrow_mut().set_text(&run_instr.to_string());
        // Cycles.
        self.cycles
            .borrow_mut()
            .set_text(&format!("T-cycles: {}   M-cycles: {}", t_cycles, m_cycles));

        // Halted.
        if halted {
            self.status.borrow_mut().set_text("HALT");
            self.status.borrow_mut().set_color(RED);
        } else {
            self.status.borrow_mut().set_text("RUN");
            self.status.borrow_mut().set_color(GREEN);
        }

        // Registers.
        self.regs.borrow_mut().set_text(&format!(
            "AF: 0x{:02X}{:02X} BC: 0x{:02X}{:02X}",
            reg.a, reg.f, reg.b, reg.c,
        ));
        self.regs2.borrow_mut().set_text(&format!(
            "DE: 0x{:02X}{:02X}  HL: 0x{:02X}{:02X}  SP: 0x{:04X}",
            reg.d, reg.e, reg.h, reg.l, reg.sp
        ));
    }

    /// Sets the visibility of the debug interface.
    pub fn set_debug_visibility(&mut self, visible: bool) {
        self.main_layout.borrow_mut().set_visible(visible);
    }

    /// The debug widgets float beside the LCD
    /// screen, so we need to update their positions.
    pub fn update_positions(&mut self, ui: &UIManager, dx: f32, dy: f32) {
        self.main_layout
            .borrow_mut()
            .layout(ui, dx + 10.0, dy + 10.0);
    }
}
