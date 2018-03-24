use CowStr;
use indexmap::IndexMap;
use std::fmt::{self, Formatter};
use std::fmt::Display;
#[cfg(target_arch = "wasm32")]
use stdweb::web::Element;
#[cfg(target_arch = "wasm32")]
use events::DOMEvent;
use vnode::VNode;
#[cfg(not(target_arch = "wasm32"))]
use traits::ServerRender;

#[derive(Debug, Eq, PartialEq)]
pub struct ClassString(CowStr);

impl Display for ClassString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, " class=\"{}\"", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Attributes(IndexMap<CowStr, CowStr>);

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (k, v) in self.0.iter() {
            write!(f, " {}=\"{}\"", k, v)?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Eq, PartialEq)]
pub struct Events(Vec<Box<DOMEvent>>);

#[derive(Debug, Eq, PartialEq)]
pub struct VElement {
    tag: CowStr,
    class: Option<ClassString>,
    attrs: Option<Attributes>,
    child: Option<Box<VNode>>,
    is_self_closing: bool,
    #[cfg(target_arch = "wasm32")]
    events: Events,
    #[cfg(target_arch = "wasm32")]
    dom_ref: Option<Element>,
}

impl VElement {
    pub fn new(tag: CowStr, class: Option<ClassString>, attrs: Option<Attributes>, child: Option<VNode>, is_self_closing: bool) -> VElement {
        VElement {
            // TODO: validate tag string first
            tag,
            class,
            attrs,
            child: child.map(|it| Box::new(it)),
            is_self_closing,
            #[cfg(target_arch = "wasm32")]
            events: Events(vec![]),
            #[cfg(target_arch = "wasm32")]
            dom_ref: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn dom_ref(&self) -> Option<&Element> {
        self.dom_ref.as_ref()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_events(&mut self, events: Vec<Box<DOMEvent>>) {
        self.events.0 = events;
    }
}

impl Display for VElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<{}", self.tag)?;
        if let Some(ref class) = self.class {
            write!(f, "{}", class)?;
        }
        if let Some(ref attrs) = self.attrs {
            write!(f, "{}", attrs)?;
        }
        if self.is_self_closing {
            write!(f, ">")
        } else {
            write!(f, ">")?;
            if let Some(ref child) = self.child {
                write!(f, "{}", child)?;
            }
            write!(f, "</{}>", self.tag)
        }
    }
}

impl<A: Into<CowStr>> From<A> for ClassString {
    fn from(item: A) -> Self {
        ClassString(item.into())
    }
}

impl<A, B> From<Vec<(A, B)>> for Attributes where
    A: Into<CowStr>,
    B: Into<CowStr> {
    fn from(item: Vec<(A, B)>) -> Self {
        Attributes(item.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect())
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, VNode, bool)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, VNode, bool)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, Some(item.2), item.3)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, VNode)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, VNode)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, Some(item.2), false)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, bool)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, bool)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, None, item.2)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>)) -> Self {
        let tag = item.0.into();
        let (class, attrs) = split_into_class_and_attrs(item.1.into());
        VElement::new(tag, class, attrs, None, false)
    }
}

impl<A> From<(A, bool)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, bool)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, None, item.1)
    }
}

impl<A> From<(A, ())> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, ())) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, None, false)
    }
}

impl<A> From<(A, VNode, bool)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, VNode, bool)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, Some(item.1), item.2)
    }
}

impl<A> From<(A, VNode)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, VNode)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, Some(item.1), false)
    }
}

fn split_into_class_and_attrs(mut attrs: Attributes) -> (Option<ClassString>, Option<Attributes>) {
    let class = attrs.0.swap_remove("class").map(|it| it.into());
    (class, if attrs.0.len() == 0 { None } else { Some(attrs) })
}

#[cfg(not(target_arch = "wasm32"))]
impl ServerRender for VElement {
    fn server_render(&mut self) {
        if let Some(ref mut child) = self.child {
            child.server_render();
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use stdweb::web::{Element, document, INode, IElement};
    use vdiff::{DOMPatch, DOMRemove};
    use super::{VElement, ClassString, Attributes, Events};
    use vdiff::DOMReorder;
    use vdiff::DOMNode;
    use stdweb::web::Node;
    use traits::DOMRender;
    use events::RenderRequestSender;

    impl DOMPatch<VElement> for VElement {
        fn patch(mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<VElement>, render_req: RenderRequestSender) -> Self {
            if let Some(old_vnode) = old_vnode {
                if old_vnode.tag != self.tag {
                    old_vnode.remove(parent);
                    create_new_dom_node(self, parent, next, render_req)
                } else {
                    let el = old_vnode.dom_ref().expect("Older element must have dom_ref")
                        .clone();
                    self.class = self.class.patch(&el, None, old_vnode.class, render_req.clone());
                    self.attrs = self.attrs.patch(&el, None, old_vnode.attrs, render_req.clone());
                    self.child = self.child.patch(&el, None, old_vnode.child.map(|it| *it), render_req.clone());
                    self.events = self.events.patch(&el, None, Some(old_vnode.events), render_req);
                    self.dom_ref = Some(el);
                    self
                }
            } else {
                create_new_dom_node(self, parent, next, render_req)
            }
        }
    }

    impl DOMReorder for VElement {
        fn move_to_last(&self, parent: &Element) {
            let dom_ref = self.dom_ref().expect("Cannot append previously non-existent element.");
            parent.append_child(dom_ref);
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            parent.insert_before(self.dom_ref().expect("Cannot insert previously non-existent text node."), next)
                .unwrap();
        }
    }

    impl DOMRemove for VElement {
        fn remove(mut self, parent: &Element) {
            let dom_ref = self.dom_ref.take()
                .expect("Cannot remove non-existent element.");
            // Dismember the events
            self.events.remove(&dom_ref);
            // Remove child and their events
            if let Some(child) = self.child {
                child.remove(&dom_ref);
            }
            // Lastly remove self
            parent.remove_child(&dom_ref).unwrap();
        }
    }

    fn create_new_dom_node(mut vel: VElement, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender) -> VElement {
        let el_node = document().create_element(&vel.tag).unwrap();
        vel.class = vel.class.patch(&el_node, None, None, render_req.clone());
        vel.attrs = vel.attrs.patch(&el_node, None, None, render_req.clone());
        vel.child = vel.child.patch(&el_node, None, None, render_req.clone());
        vel.events = vel.events.patch(&el_node, None, None, render_req);
        if let Some(next) = next {
            parent.insert_before(&el_node, next).unwrap();
        } else {
            parent.append_child(&el_node);
        }
        vel.dom_ref = Some(el_node);
        vel
    }

    impl DOMPatch<ClassString> for ClassString {
        fn patch(self, parent: &Element, _: Option<&Node>, old_value: Option<ClassString>, _: RenderRequestSender) -> Self {
            if Some(&self) != old_value.as_ref() {
                parent.set_attribute("class", &self.0)
                    .unwrap();
            }
            self
        }
    }

    impl DOMRemove for ClassString {
        fn remove(self, parent: &Element) {
            parent.remove_attribute("class");
        }
    }

    impl DOMPatch<Attributes> for Attributes {
        fn patch(self, parent: &Element, _: Option<&Node>, old_vnode: Option<Attributes>, _: RenderRequestSender) -> Self {
            if let Some(mut old_attributes) = old_vnode {
                for (k, v) in self.0.clone() {
                    let old_attr_val = old_attributes.0.swap_remove(&k);
                    if Some(&v) != old_attr_val.as_ref() {
                        parent.set_attribute(&k, &v).unwrap();
                    }
                }
                for (k, _) in old_attributes.0.iter() {
                    parent.remove_attribute(&k);
                }
            } else {
                for (k, v) in self.0.clone() {
                    parent.set_attribute(&k, &v).unwrap();
                }
            }
            self
        }
    }

    impl DOMRemove for Attributes {
        fn remove(self, parent: &Element) {
            for (k, _) in self.0.iter() {
                parent.remove_attribute(k);
            }
        }
    }

    impl DOMPatch<Events> for Events {
        fn patch(mut self, parent: &Element, _: Option<&Node>, old_vnode: Option<Events>, _: RenderRequestSender) -> Self {
            // Remove older events because their is no way for Eq between two events.
            old_vnode.remove(parent);
            for ev in self.0.iter_mut() {
                ev.attach(parent);
            }
            self
        }
    }

    impl DOMRemove for Events {
        fn remove(mut self, _: &Element) {
            for ev in self.0.iter_mut() {
                ev.detach();
            }
        }
    }

    impl DOMNode for VElement {
        fn dom_node(&self) -> Option<Node> {
            self.dom_ref.clone().map(|it| it.into())
        }
    }

    impl DOMRender for VElement {
        fn dom_render(&mut self, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender) {
            if let Some(ref mut child) = self.child {
                child.dom_render(parent, next, render_req);
            }
        }
    }
}
