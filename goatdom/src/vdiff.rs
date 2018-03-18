use stdweb::web::Element;
use vnode::VNode;

/// Required to update the DOM on the `parent` node. It is also tasked with Diffing along
/// as it creates patches.
pub trait DOMPatch<T> {
    fn patch(&mut self, parent: &Element, old_vnode: Option<&T>);
}

/// Required when removing stale `VNodes`.
pub trait DOMRemove {
    fn remove(&self, parent: &Element);
}

/// Required when re-ordering the `VList` children. Reordering is done by appending the dom node
/// again in a new order.
pub trait DOMReorder {
    fn reorder(&self, parent: &Element);
}

impl<T, Q> DOMPatch<T> for Option<Q> where
    Q: DOMPatch<T>,
    T: DOMRemove {
    fn patch(&mut self, parent: &Element, old_vnode: Option<&T>) {
        if let Some(ref mut this) = *self {
            this.patch(parent, old_vnode);
        } else {
            old_vnode.remove(parent);
        }
    }
}

impl<T, Q> DOMPatch<Q> for Box<T> where
    T: DOMPatch<Q> {
    fn patch(&mut self, parent: &Element, old_vnode: Option<&Q>) {
        let this = &mut **self;
        this.patch(parent, old_vnode);
    }
}

impl<'a, T: DOMRemove> DOMRemove for Option<&'a T> {
    fn remove(&self, parent: &Element) {
        if let Some(ref inner) = *self {
            inner.remove(parent);
        }
    }
}