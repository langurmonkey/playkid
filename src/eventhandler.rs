use sdl2::event::Event;
/// Event handler trait.
pub trait EventHandler {
    fn handle_event(&mut self, event: &Event) -> bool;
}
