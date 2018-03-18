use stdweb::web::Element;
use vnode::VNode;

pub trait VDiff {
    type VNodeLike;

    fn apply(&mut self, parent: &Element, old_vnode: Option<&Self::VNodeLike>);
}