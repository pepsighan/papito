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

    fn state_changed(&self) -> bool {
        *self.state_changed.borrow()
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
            if let Some(ref mut rendered) = self.rendered {
                rendered.remove(parent);
            }
        }
    }

    impl DOMReorder for VComponent {
        fn move_to_last(&self, parent: &Element) {
            if let Some(ref rendered) = self.rendered {
                rendered.move_to_last(parent);
            }
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            if let Some(ref rendered) = self.rendered {
                rendered.move_before(parent, next);
            }
        }
    }

    impl DOMNode for VComponent {
        fn dom_node(&self) -> Option<Node> {
            self.rendered.as_ref().and_then(|it| it.dom_node())
        }
    }

    impl InternalRender for VComponent {
        fn internal_render(&mut self, parent: &Element, next: Option<&Node>) {
            if self.instance.is_none() {
                self.init();
            }
            if self.rendered.is_none() {
                // First time being rendered
                let instance = self.instance.as_mut().unwrap();
                let mut rendered = instance.render();
                rendered.patch(parent, next, None);
                self.rendered = Some(Box::new(rendered));
                instance.mounted();
            } else {
                if self.state_changed() {
                    // TODO: Support props
                    let mut old_rendered = self.rendered.take().unwrap();
                    let instance = self.instance.as_mut().unwrap();
                    let mut newly_rendered = instance.render();
                    newly_rendered.patch(parent, next, Some(&mut *old_rendered));
                    self.rendered = Some(Box::new(newly_rendered));
                    instance.updated();
                } else {
                    // No change. Propagate till a changed/new component is found
                    self.rendered.as_mut().unwrap().internal_render(parent, next);
                }
            }
        }
    }
}