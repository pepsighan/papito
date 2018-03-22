use vnode::VNode;
#[cfg(target_arch = "wasm32")]
use stdweb::web::{Element, Node};

#[cfg(target_arch = "wasm32")]
pub trait InternalRender {
    fn internal_render(&mut self, parent: &Element, next: Option<&Node>);
}

pub trait Component: Lifecycle {
    fn create(notifier: Box<Fn()>) -> Self;
}

pub trait Lifecycle: Render {
    fn created(&mut self);

    fn mounted(&mut self);

    fn before_update(&mut self);

    fn updated(&mut self);

    fn destroyed(&mut self);
}

pub trait Render {
    fn render(&self) -> VNode;
}