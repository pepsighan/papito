use vnode::VNode;
#[cfg(target_arch = "wasm32")]
use stdweb::web::{Element, Node};

#[cfg(target_arch = "wasm32")]
pub trait InternalRender {
    fn internal_render(&mut self, parent: &Element, next: Option<&Node>);
}