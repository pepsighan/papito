use stdweb::web::Element;

/// Add or remove events from the DOM
pub trait DOMEvents {
    fn attach(&mut self, parent: &Element);

    fn detach(&mut self);
}