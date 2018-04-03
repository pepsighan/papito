use vnode::VNode;
#[cfg(target_arch = "wasm32")]
use stdweb::web::{Element, Node};
#[cfg(target_arch = "wasm32")]
use events::RenderRequestSender;
use std::any::Any;

#[cfg(target_arch = "wasm32")]
pub trait DOMRender {
    fn dom_render(&mut self, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender);
}

#[cfg(not(target_arch = "wasm32"))]
pub trait ServerRender {
    fn server_render(&mut self);
}

#[cfg(not(target_arch = "wasm32"))]
pub trait RenderToString {
    fn render_to_string(&mut self) -> String;
}

pub trait Component: Lifecycle {
    type Props: PartialEq;

    fn create(props: Self::Props, notifier: Box<Fn()>) -> Self;

    fn update(&self, props: Self::Props);

    fn eq_props(&self, rhs: &Self::Props) -> bool;
}

pub trait Lifecycle: Render + AsAny {
    fn created(&self) {}

    fn mounted(&self) {}

    fn updated(&self) {}

    fn destroyed(&self) {}
}

pub trait Render {
    fn render(&self) -> VNode;
}

pub trait AsAny {
    fn as_any(&self) -> &Any;
}

impl<T: Lifecycle + 'static> AsAny for T {
    fn as_any(&self) -> &Any {
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: ServerRender + ToString> RenderToString for T {
    fn render_to_string(&mut self) -> String {
        self.server_render();
        self.to_string()
    }
}
