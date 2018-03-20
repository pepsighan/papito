use stdweb::web::Element;
use stdweb::web::Node;

/// Required to update the DOM on the `parent` node. It is also tasked with Diffing along
/// as it creates patches.
pub trait DOMPatch<T> {
    fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<&mut T>);
}

/// Required when removing stale `VNodes`.
pub trait DOMRemove {
    fn remove(&mut self, parent: &Element);
}

/// Required when re-ordering the `VList` children. Reordering is done by appending the dom node
/// again in a new order.
pub trait DOMReorder {
    fn move_to_last(&self, parent: &Element);

    fn move_before(&self, parent: &Element, next: &Node);
}

pub trait DOMNode {
    fn dom_node(&self) -> Option<Node>;
}

impl<T, Q> DOMPatch<T> for Option<Q> where
    Q: DOMPatch<T>,
    T: DOMRemove {
    fn patch(&mut self, parent: &Element, next: Option<&Node>, mut old_vnode: Option<&mut T>) {
        if let Some(ref mut this) = *self {
            this.patch(parent, next, old_vnode);
        } else {
            old_vnode.remove(parent);
        }
    }
}

impl<T, Q> DOMPatch<Q> for Box<T> where
    T: DOMPatch<Q> {
    fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<&mut Q>) {
        let this = &mut **self;
        this.patch(parent, next, old_vnode);
    }
}

impl<'a, T: DOMRemove> DOMRemove for Option<&'a mut T> {
    fn remove(&mut self, parent: &Element) {
        if let Some(ref mut inner) = *self {
            inner.remove(parent);
        }
    }
}