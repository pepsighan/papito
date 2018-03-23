use vnode::VNode;
use std::any::TypeId;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;
use traits::Component;
use traits::Lifecycle;
#[cfg(not(target_arch = "wasm32"))]
use traits::ServerRender;
use events::RenderRequestSender;

pub struct VComponent {
    type_id: TypeId,
    instance: Option<Box<Lifecycle>>,
    #[cfg(target_arch = "wasm32")]
    initializer: Box<Fn(RenderRequestSender) -> Box<Lifecycle>>,
    #[cfg(not(target_arch = "wasm32"))]
    initializer: Box<Fn() -> Box<Lifecycle>>,
    rendered: Option<Box<VNode>>,
    state_changed: Rc<RefCell<bool>>,
}

impl VComponent {
    #[cfg(target_arch = "wasm32")]
    pub fn new<T: Component + 'static>() -> VComponent {
        let state_changed = Rc::new(RefCell::new(false));
        let state_changed_writer = state_changed.clone();
        VComponent {
            type_id: TypeId::of::<T>(),
            instance: None,
            initializer: Box::new(move |render_req| {
                let state_changed = state_changed_writer.clone();
                let notifier = Box::new(move || {
                    *state_changed.borrow_mut() = true;
                    render_req.send();
                });
                Box::new(T::create(notifier))
            }),
            rendered: None,
            state_changed,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    fn init(&mut self, render_req: RenderRequestSender) {
        let initializer = &self.initializer;
        let mut instance = initializer(render_req);
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

#[cfg(not(target_arch = "wasm32"))]
impl ServerRender for VComponent {
    fn server_render(&mut self) {
        debug_assert!(self.instance.is_none());
        debug_assert!(self.rendered.is_none());
        self.init();
        let instance = self.instance.as_mut().unwrap();
        let mut rendered = instance.render();
        rendered.server_render();
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
    use traits::DOMRender;
    use events::RenderRequestSender;

    impl DOMPatch<VComponent> for VComponent {
        fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<&mut VComponent>, render_req: RenderRequestSender) {
            // Those that are new here, are unrendered and those old require re-rendering
            if let Some(old_comp) = old_vnode {
                if self.type_id == old_comp.type_id {
                    // Throw out the newer component and reuse older
                    // TODO: Push updated props
                    old_comp.dom_render(parent, next, render_req);
                } else {
                    old_comp.remove(parent);
                    create_new_component_render(self, parent, next, render_req);
                }
            } else {
                create_new_component_render(self, parent, next, render_req);
            }
        }
    }

    fn create_new_component_render(vcomp: &mut VComponent, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender) {
        debug_assert!(vcomp.instance.is_none());
        debug_assert!(vcomp.rendered.is_none());
        // Requires an initial render as they are very new
        vcomp.dom_render(parent, next, render_req);
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

    impl DOMRender for VComponent {
        fn dom_render(&mut self, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender) {
            if self.instance.is_none() {
                self.init(render_req.clone());
            }
            if self.rendered.is_none() {
                // First time being rendered
                let instance = self.instance.as_mut().unwrap();
                let mut rendered = instance.render();
                rendered.patch(parent, next, None, render_req);
                self.rendered = Some(Box::new(rendered));
                instance.mounted();
            } else {
                if self.state_changed() {
                    // TODO: Support props
                    let mut old_rendered = self.rendered.take().unwrap();
                    let instance = self.instance.as_mut().unwrap();
                    let mut newly_rendered = instance.render();
                    newly_rendered.patch(parent, next, Some(&mut *old_rendered), render_req);
                    self.rendered = Some(Box::new(newly_rendered));
                    instance.updated();
                } else {
                    // No change. Propagate till a changed/new component is found
                    self.rendered.as_mut().unwrap().dom_render(parent, next, render_req);
                }
            }
        }
    }
}