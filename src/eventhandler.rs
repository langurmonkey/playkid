use winit_input_helper::WinitInputHelper;
/// Event handler trait.
pub trait EventHandler {
    fn handle_event(&mut self, event: WinitInputHelper) -> bool;
}
