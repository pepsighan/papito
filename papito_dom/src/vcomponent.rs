use vnode::VNode;
use std::any::TypeId;
use std::fmt::Display;
use std::fmt::{Formatter, self};

#[derive(Debug)]
pub struct VComponent {
    type_id: TypeId,
    instance: Box<Lifecycle>,
    rendered: Option<Box<VNode>>,
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

impl Eq for VComponent {}

impl PartialEq for VComponent {
    fn eq(&self, other: &VComponent) -> bool {
        self.type_id == other.type_id &&
            self.rendered == other.rendered
    }
}

impl Display for VComponent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(ref rendered) = self.rendered {
            write!(f, "{}", rendered)
        } else {
            Ok(())
        }
    }
}