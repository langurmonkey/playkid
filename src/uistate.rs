/// # UI state
/// UI action queue to send requests to the main app.
pub struct UIState {
    pub exit_requested: bool,
    pub screenshot_requested: bool,
    pub is_picking_file: bool,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            exit_requested: false,
            screenshot_requested: false,
            is_picking_file: false,
        }
    }
}
