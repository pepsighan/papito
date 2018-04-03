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
#[cfg(target_arch = "wasm32")]
use events::RenderRequestSender;
use std::mem;

struct Props;

pub struct VComponent {
    type_id: TypeId,
    instance: Option<Box<Lifecycle>>,
    props: Option<*mut Props>,
    #[cfg(target_arch = "wasm32")]
    initializer: Box<Fn(*mut Props, RenderRequestSender) -> Box<Lifecycle>>,
    #[cfg(not(target_arch = "wasm32"))]
    initializer: Box<Fn(*mut Props) -> Box<Lifecycle>>,
    #[cfg(target_arch = "wasm32")]
    props_setter: Box<Fn(&mut Box<Lifecycle>, *mut Props)>,
    rendered: Option<Box<VNode>>,
    #[cfg(target_arch = "wasm32")]
    state_changed: Rc<RefCell<bool>>,
}

impl VComponent {
    #[cfg(target_arch = "wasm32")]
    pub fn new<T: Component + 'static>(props: T::Props) -> VComponent {
        let state_changed = Rc::new(RefCell::new(false));
        let state_changed_writer = state_changed.clone();
        let props: *mut Props = unsafe {
            mem::transmute(Box::into_raw(Box::new(props)))
        };
        VComponent {
            type_id: TypeId::of::<T>(),
            instance: None,
            props: Some(props),
            initializer: Box::new(move |props, render_req| {
                let state_changed = state_changed_writer.clone();
                let notifier = Box::new(move || {
                    *state_changed.borrow_mut() = true;
                    render_req.send();
                });
                let props: T::Props = unsafe {
                    *Box::from_raw(mem::transmute(props))
                };
                Box::new(T::create(props, notifier))
            }),
            props_setter: Box::new(|instance, props| {
                let props: T::Props = unsafe {
                    *Box::from_raw(mem::transmute(props))
                };
                let instance = instance.as_any().downcast_ref::<T>()
                    .expect("Impossible. The instance cannot be of any other type");
                let is_diff = instance.eq_props(&props);
                if is_diff {
                    T::update(instance, props);
                }
            }),
            rendered: None,
            state_changed,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<T: Component + 'static>(props: T::Props) -> VComponent {
        let state_changed = Rc::new(RefCell::new(false));
        let state_changed_writer = state_changed.clone();
        let props: *mut Props = unsafe {
            mem::transmute(Box::into_raw(Box::new(props)))
        };
        VComponent {
            type_id: TypeId::of::<T>(),
            instance: None,
            props: Some(props),
            initializer: Box::new(move |props| {
                let state_changed = state_changed_writer.clone();
                let notifier = Box::new(move || {
                    *state_changed.borrow_mut() = true;
                });
                let props: T::Props = unsafe {
                    *Box::from_raw(mem::transmute(props))
                };
                Box::new(T::create(props, notifier))
            }),
            rendered: None
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn init(&mut self, render_req: RenderRequestSender) {
        let initializer = &self.initializer;
        let props = self.props.take().expect("Impossible. The props are always provided");
        let instance = initializer(props, render_req);
        instance.created();
        self.instance = Some(instance);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn init(&mut self) {
        let initializer = &self.initializer;
        let props = self.props.take().expect("Impossible. The props are always provided");
        let instance = initializer(props);
        instance.created();
        self.instance = Some(instance);
    }

    // Only use this when the Type of the props is same as that of this Component's props
    #[cfg(target_arch = "wasm32")]
    unsafe fn set_props(&mut self, props: *mut Props) {
        debug_assert!(self.instance.is_some());
        let props_setter = &self.props_setter;
        props_setter(self.instance.as_mut().unwrap(), props);
    }

    #[cfg(target_arch = "wasm32")]
    fn take_props(&mut self) -> *mut Props {
        self.props.take()
            .expect("Props already taken")
    }

    #[cfg(target_arch = "wasm32")]
    fn state_changed(&self) -> bool {
        *self.state_changed.borrow()
    }

    #[cfg(target_arch = "wasm32")]
    fn unset_state_changed(&self) {
        *self.state_changed.borrow_mut() = false;
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
        fn patch(mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<VComponent>, render_req: RenderRequestSender) -> Self {
            // Those that are new here, are unrendered and those old require re-rendering
            if let Some(mut old_comp) = old_vnode {
                if self.type_id == old_comp.type_id {
                    // Throw out the newer component, reuse older and pass the newer props
                    unsafe {
                        // Safe to use because both the props are of same type as both
                        // components are of same type
                        old_comp.set_props(self.take_props());
                    }
                    old_comp.dom_render(parent, next, render_req);
                    old_comp
                } else {
                    old_comp.remove(parent);
                    create_new_component_render(&mut self, parent, next, render_req);
                    self
                }
            } else {
                create_new_component_render(&mut self, parent, next, render_req);
                self
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
        fn remove(mut self, parent: &Element) {
            debug_assert!(self.instance.is_some());
            debug_assert!(self.rendered.is_some());
            self.rendered.unwrap().remove(parent);
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
                let rendered = instance.render();
                let rendered = rendered.patch(parent, next, None, render_req);
                self.rendered = Some(Box::new(rendered));
                instance.mounted();
            } else {
                if self.state_changed() {
                    self.unset_state_changed();
                    let old_rendered = self.rendered.take().unwrap();
                    let instance = self.instance.as_mut().unwrap();
                    let newly_rendered = instance.render();
                    let newly_rendered = newly_rendered.patch(parent, next, Some(*old_rendered), render_req);
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