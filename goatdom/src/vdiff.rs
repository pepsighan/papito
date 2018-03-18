use stdweb::web::Element;
use vnode::VNode;

pub trait VDiff {
    fn apply(&mut self, parent: &Element, old_vnode: &VNode);
}