use vnode::VNode;
use std::any::TypeId;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use std::fmt::Debug;
use traits::InternalRender;

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

impl Debug for VComponent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(ref rendered) = self.rendered {
            write!(f, "{:?}", rendered)
        } else {
            Ok(())
        }
    }
}

impl InternalRender for VComponent {
    fn internal_render(self) -> VNode {
        unimplemented!()
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use vdiff::DOMPatch;
    use vcomponent::VComponent;
    use stdweb::web::Element;
    use stdweb::web::Node;
    use vdiff::DOMRemove;
    use vdiff::DOMReorder;
    use vdiff::DOMNode;

    impl DOMPatch<VComponent> for VComponent {
        fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<&mut VComponent>) {
            unimplemented!()
        }
    }

    impl DOMRemove for VComponent {
        fn remove(&mut self, parent: &Element) {
            unimplemented!()
        }
    }

    impl DOMReorder for VComponent {
        fn move_to_last(&self, parent: &Element) {
            unimplemented!()
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            unimplemented!()
        }
    }

    impl DOMNode for VComponent {
        fn dom_node(&self) -> Option<Node> {
            unimplemented!()
        }
    }
}
