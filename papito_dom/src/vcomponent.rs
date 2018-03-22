use vnode::VNode;
use std::any::TypeId;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;

pub struct VComponent {
    type_id: TypeId,
    instance: Option<Box<Lifecycle>>,
    initializer: Box<Fn() -> Box<Lifecycle>>,
    rendered: Option<Box<VNode>>,
    state_changed: Rc<RefCell<bool>>,
}

pub trait Component: Lifecycle {
    fn create() -> Self;

    fn update(&mut self);

    fn destroy(&mut self);
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

impl VComponent {
    pub fn new<T: Component + 'static>() -> VComponent {
        VComponent {
            type_id: TypeId::of::<T>(),
            instance: None,
            initializer: Box::new(|| {
                Box::new(T::create())
            }),
            rendered: None,
            state_changed: Rc::new(RefCell::new(false)),
        }
    }

    fn init(&mut self) {
        let initializer = &self.initializer;
        let mut instance = initializer();
        instance.created();
        self.instance = Some(instance);
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

#[cfg(target_arch = "wasm32")]
mod wasm {
    use vdiff::DOMPatch;
    use vcomponent::VComponent;
    use stdweb::web::Element;
    use stdweb::web::Node;
    use vdiff::DOMRemove;
    use vdiff::DOMReorder;
    use vdiff::DOMNode;
    use traits::InternalRender;

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

    impl InternalRender for VComponent {
        fn internal_render(&mut self, parent: &Element, next: Option<&Node>) {
            if self.instance.is_none() {
                self.init();
            }
            let instance = self.instance.as_mut().unwrap();
            if self.rendered.is_none() {
                let mut rendered = instance.render();
                rendered.patch(parent, next, None);
            }
        }
    }
}