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
    use stdweb::web::INode;
    use vdiff::DOMReorder;

    macro_rules! match_for_vnode_patch {
        ($against:ident, $parent:ident, $old_vnode:ident, [$( $variant:ident ),*] ) => {
            match *$against {
                $(
                    VNode::$variant(ref mut node_like) => {
                        if let Some(&mut VNode::$variant(ref mut old_node_like)) = $old_vnode {
                            node_like.patch($parent, Some(old_node_like));
                        } else {
                            $old_vnode.remove($parent);
                            node_like.patch($parent, None);
                        }
                    }
                )*
            }
        };
    }

    impl DOMPatch<VNode> for VNode {
        fn patch(&mut self, parent: &Element, mut old_vnode: Option<&mut VNode>) {
            match_for_vnode_patch!(self, parent, old_vnode, [Text, Element, List]);
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
        fn append_child(&self, parent: &Element) {
            match *self {
                VNode::Text(ref text) => text.append_child(parent),
                VNode::Element(ref element) => element.append_child(parent),
                VNode::List(ref list) => list.append_child(parent)
            }
        }

        fn insert_before<T: INode>(&self, parent: &Element, next: &T) {
            match *self {
                VNode::Text(ref text) => text.insert_before(parent, next),
                VNode::Element(ref element) => element.insert_before(parent, next),
                VNode::List(ref list) => list.insert_before(parent, next)
            }
        }
    }
}