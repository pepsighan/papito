use velement::VElement;
use vlist::VList;
use vtext::VText;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use vcomponent::VComponent;
use traits::ServerRender;

#[derive(Debug, Eq, PartialEq)]
pub enum VNode {
    Text(VText),
    Element(VElement),
    List(VList),
    Component(VComponent)
}

impl VNode {
    pub fn new<T: Into<VNode>>(content: T) -> VNode {
        content.into()
    }
}

impl Display for VNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            VNode::Text(ref text) => write!(f, "{}", text),
            VNode::Element(ref element) => write!(f, "{}", element),
            VNode::List(ref list) => write!(f, "{}", list),
            VNode::Component(ref component) => write!(f, "{}", component)
        }
    }
}

macro_rules! impl_conversion_to_vnode {
    ($variant:ident, $inner:ty) => {
        impl From<$inner> for VNode {
            fn from(item: $inner) -> Self {
                VNode::$variant(item)
            }
        }
    };
}

impl_conversion_to_vnode!(Text, VText);
impl_conversion_to_vnode!(Element, VElement);
impl_conversion_to_vnode!(List, VList);
impl_conversion_to_vnode!(Component, VComponent);

impl ServerRender for VNode {
    fn server_render(&mut self) {
        match *self {
            VNode::Component(ref mut component) => component.server_render(),
            VNode::List(ref mut list) => list.server_render(),
            VNode::Element(ref mut element) => element.server_render(),
            VNode::Text(_) => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use vdiff::{DOMPatch, DOMRemove};
    use stdweb::web::Element;
    use super::VNode;
    use vdiff::DOMReorder;
    use vdiff::DOMNode;
    use stdweb::web::Node;
    use traits::DOMRender;

    macro_rules! match_for_vnode_patch {
        ($against:ident, $parent:ident, $next:ident, $old_vnode:ident, [$( $variant:ident ),*] ) => {
            match *$against {
                $(
                    VNode::$variant(ref mut node_like) => {
                        if let Some(&mut VNode::$variant(ref mut old_node_like)) = $old_vnode {
                            node_like.patch($parent, $next, Some(old_node_like));
                        } else {
                            $old_vnode.remove($parent);
                            node_like.patch($parent, $next, None);
                        }
                    }
                )*
            }
        };
    }

    impl DOMPatch<VNode> for VNode {
        fn patch(&mut self, parent: &Element, next: Option<&Node>, mut old_vnode: Option<&mut VNode>) {
            match_for_vnode_patch!(self, parent, next, old_vnode, [Text, Element, List, Component]);
        }
    }

    impl DOMRemove for VNode {
        fn remove(&mut self, parent: &Element) {
            match *self {
                VNode::Text(ref mut text) => text.remove(parent),
                VNode::Element(ref mut element) => element.remove(parent),
                VNode::List(ref mut list) => list.remove(parent),
                VNode::Component(ref mut component) => component.remove(parent)
            }
        }
    }

    impl DOMReorder for VNode {
        fn move_to_last(&self, parent: &Element) {
            match *self {
                VNode::Text(ref text) => text.move_to_last(parent),
                VNode::Element(ref element) => element.move_to_last(parent),
                VNode::List(ref list) => list.move_to_last(parent),
                VNode::Component(ref component) => component.move_to_last(parent)
            }
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            match *self {
                VNode::Text(ref text) => text.move_before(parent, next),
                VNode::Element(ref element) => element.move_before(parent, next),
                VNode::List(ref list) => list.move_before(parent, next),
                VNode::Component(ref component) => component.move_before(parent, next)
            }
        }
    }

    impl DOMNode for VNode {
        fn dom_node(&self) -> Option<Node> {
            match *self {
                VNode::Text(ref text) => text.dom_node(),
                VNode::Element(ref element) => element.dom_node(),
                VNode::List(ref list) => list.dom_node(),
                VNode::Component(ref component) => component.dom_node()
            }
        }
    }

    impl DOMRender for VNode {
        fn dom_render(&mut self, parent: &Element, next: Option<&Node>) {
            match *self {
                VNode::Component(ref mut component) => component.dom_render(parent, next),
                VNode::List(ref mut list) => list.dom_render(parent, next),
                VNode::Element(ref mut element) => element.dom_render(parent, next),
                VNode::Text(_) => {}
            }
        }
    }
}
