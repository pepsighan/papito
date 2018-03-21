use vnode::VNode;
use std::any::TypeId;

pub struct VComponent {
    type_id: TypeId,
    instance: Box<Lifecycle>,
    rendered: Option<VNode>,
}

pub trait Component: Lifecycle + Render {
    type Props;

    fn create(props: Self::Props) -> Self;

    fn update(&mut self);

    fn destroy(&mut self);
}

pub trait Lifecycle {
    fn created(&mut self);

    fn mounted(&mut self);

    fn before_update(&mut self);

    fn updated(&mut self);

    fn destroyed(&mut self);
}

pub trait Render {
    fn render(&self) -> VNode;
}

impl VComponent {
    pub fn new<T: Component + 'static>(props: T::Props) -> VComponent {
        let mut comp = T::create(props);
        comp.created();
        VComponent {
            type_id: TypeId::of::<T>(),
            instance: Box::new(comp),
            rendered: None
        }
    }
}