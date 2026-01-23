/// UI action queue.
pub struct UIState {
    pub exit_requested: bool,
    pub screenshot_requested: bool,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            exit_requested: false,
            screenshot_requested: false,
        }
    }
}
