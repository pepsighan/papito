use velement::VElement;
use vlist::VList;
use vtext::VText;
use std::fmt::Display;
use std::fmt::{Formatter, self};
#[cfg(target_arch = "wasm32")]
use vdiff::{DOMPatch, DOMRemove, DOMReorder};
#[cfg(target_arch = "wasm32")]
use stdweb::web::Element;

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

macro_rules! match_for_vnode_patch {
    ($against:ident, $parent:ident, $old_vnode:ident, [$( $variant:ident ),*] ) => {
        match *$against {
            $(
                VNode::$variant(ref mut node_like) => {
                    if let Some(&VNode::$variant(ref old_node_like)) = $old_vnode {
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

#[cfg(target_arch = "wasm32")]
impl DOMPatch<VNode> for VNode {
    fn patch(&mut self, parent: &Element, old_vnode: Option<&VNode>) {
        match_for_vnode_patch!(self, parent, old_vnode, [Text, Element, List]);
    }
}

#[cfg(target_arch = "wasm32")]
impl DOMRemove for VNode {
    fn remove(&self, parent: &Element) {
        match *self {
            VNode::Text(ref text) => text.remove(parent),
            VNode::Element(ref element) => element.remove(parent),
            VNode::List(ref list) => list.remove(parent)
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl DOMReorder for VNode {
    fn reorder(&self, parent: &Element) {
        match *self {
            VNode::Text(ref text) => text.reorder(parent),
            VNode::Element(ref element) => element.reorder(parent),
            VNode::List(ref list) => list.reorder(parent)
        }
    }
}