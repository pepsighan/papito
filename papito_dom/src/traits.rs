use vnode::VNode;

pub trait InternalRender {
    fn internal_render(self) -> VNode;
}