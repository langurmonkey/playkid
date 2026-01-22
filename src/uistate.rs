/// UI action queue.
pub struct UIState {
    pub reset_requested: bool,
    pub step_requested: bool,
    pub scanline_requested: bool,
    pub continue_requested: bool,
    pub debug_requested: bool,
    pub br_add_requested: bool,
    pub br_remove_requested: bool,
    pub br_clear_requested: bool,
    pub br_addr: u16,
    pub exit_requested: bool,
    pub fps_requested: bool,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            reset_requested: false,
            step_requested: false,
            scanline_requested: false,
            continue_requested: false,
            debug_requested: false,
            br_add_requested: false,
            br_remove_requested: false,
            br_clear_requested: false,
            br_addr: 0x00,
            exit_requested: false,
            fps_requested: false,
        }
    }
}
