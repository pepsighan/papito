use stdweb::web::{Element, EventListenerHandle};
use stdweb::web::event::*;

/// Add or remove events from the DOM
pub trait DOMEvents {
    fn attach(&mut self, parent: &Element);

    fn detach(&mut self);
}

pub struct DOMEventListener<T, F> where
    F: FnMut(T),
    T: ConcreteEvent {
    listener: Option<F>,
    listener_handle: Option<EventListenerHandle>,
}

impl<T, F> DOMEvents for DOMEventListener<T, F> {
    fn attach(&mut self, parent: &Element) {
        let listener = self.listener.take()
            .expect("Event listener is either already attached or detached");
        let listener_handle = parent.add_event_listener(listener);
        self.listener_handle = Some(listener_handle);
    }

    fn detach(&mut self) {
        let listener_handle = self.listener_handle.take()
            .expect("Event must be attached for it to detach");
        listener_handle.remove();
    }
}