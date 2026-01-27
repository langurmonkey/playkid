use egui::InputState;
/// Event handler trait.
pub trait EventHandler {
    fn handle_event(&mut self, i: &InputState) -> bool;
}
