use vnode::VNode;
#[cfg(target_arch = "wasm32")]
use stdweb::web::{Element, Node};
#[cfg(target_arch = "wasm32")]
use events::RenderRequestSender;

#[cfg(target_arch = "wasm32")]
pub trait DOMRender {
    fn dom_render(self, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender) -> Self;
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
    type Props;

    fn create(props: Self::Props, notifier: Box<Fn()>) -> Self;
}

pub trait Lifecycle: Render {
    fn created(&mut self) {}

    fn mounted(&mut self) {}

    fn updated(&mut self) {}

    fn destroyed(&mut self) {}
}

pub trait Render {
    fn render(&self) -> VNode;
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: ServerRender + ToString> RenderToString for T {
    fn render_to_string(&mut self) -> String {
        self.server_render();
        self.to_string()
    }
}
