use vnode::VNode;
use std::any::TypeId;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;
use traits::Component;
use traits::Lifecycle;
use traits::StringRender;

pub struct VComponent {
    type_id: TypeId,
    instance: Option<Box<Lifecycle>>,
    initializer: Box<Fn() -> Box<Lifecycle>>,
    rendered: Option<Box<VNode>>,
    state_changed: Rc<RefCell<bool>>,
}

impl VComponent {
    pub fn new<T: Component + 'static>() -> VComponent {
        let state_changed = Rc::new(RefCell::new(false));
        let state_changed_writer = state_changed.clone();
        VComponent {
            type_id: TypeId::of::<T>(),
            instance: None,
            initializer: Box::new(move || {
                let state_changed = state_changed_writer.clone();
                let notifier = Box::new(move || {
                    *state_changed.borrow_mut() = true;
                });
                Box::new(T::create(notifier))
            }),
            rendered: None,
            state_changed,
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

impl StringRender for VComponent {
    fn string_render(&mut self) {
        debug_assert!(self.instance.is_none());
        debug_assert!(self.rendered.is_none());
        self.init();
        let instance = self.instance.as_mut().unwrap();
        let mut rendered = instance.render();
        self.rendered = Some(Box::new(rendered));
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
            // Those that are new here, are unrendered and those old require re-rendering
            if let Some(old_comp) = old_vnode {
                if self.type_id == old_comp.type_id {
                    // Throw out the newer component and reuse older
                    // TODO: Push updated props
                    old_comp.internal_render(parent, next);
                } else {
                    old_comp.remove(parent);
                    create_new_component_render(self, parent, next);
                }
            } else {
                create_new_component_render(self, parent, next);
            }
        }
    }

    fn create_new_component_render(vcomp: &mut VComponent, parent: &Element, next: Option<&Node>) {
        debug_assert!(vcomp.instance.is_none());
        debug_assert!(vcomp.rendered.is_none());
        // Requires an initial render as they are very new
        vcomp.internal_render(parent, next);
    }

    impl DOMRemove for VComponent {
        fn remove(&mut self, parent: &Element) {
            debug_assert!(self.instance.is_some());
            debug_assert!(self.rendered.is_some());
            self.rendered.as_mut().unwrap().remove(parent);
            self.instance.as_mut().unwrap().destroyed();
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