use velement::VElement;
use vlist::VList;
use vtext::VText;
use std::fmt::Display;
use std::fmt::{Formatter, self};

#[derive(Debug, Eq, PartialEq)]
pub enum VNode {
    Text(VText),
    Element(VElement),
    List(VList),
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
            VNode::List(ref list) => write!(f, "{}", list)
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

#[cfg(target_arch = "wasm32")]
mod wasm {
    use vdiff::{DOMPatch, DOMRemove};
    use stdweb::web::Element;
    use super::VNode;
    use vdiff::DOMReorder;
    use vdiff::NextDOMNode;
    use stdweb::web::Node;

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
            match_for_vnode_patch!(self, parent, next, old_vnode, [Text, Element, List]);
        }
    }

    impl DOMRemove for VNode {
        fn remove(&mut self, parent: &Element) {
            match *self {
                VNode::Text(ref mut text) => text.remove(parent),
                VNode::Element(ref mut element) => element.remove(parent),
                VNode::List(ref mut list) => list.remove(parent)
            }
        }
    }

    impl DOMReorder for VNode {
        fn move_to_last(&self, parent: &Element) {
            match *self {
                VNode::Text(ref text) => text.move_to_last(parent),
                VNode::Element(ref element) => element.move_to_last(parent),
                VNode::List(ref list) => list.move_to_last(parent)
            }
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            match *self {
                VNode::Text(ref text) => text.move_before(parent, next),
                VNode::Element(ref element) => element.move_before(parent, next),
                VNode::List(ref list) => list.move_before(parent, next)
            }
        }
    }

    impl NextDOMNode for VNode {
        fn next_dom_node(&self) -> Option<Node> {
            match *self {
                VNode::Text(ref text) => text.next_dom_node(),
                VNode::Element(ref element) => element.next_dom_node(),
                VNode::List(ref list) => list.next_dom_node()
            }
        }
    }
}