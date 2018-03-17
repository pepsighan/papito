extern crate indexmap;

use std::borrow::Cow;
use vnode::VNode;

type CowStr = Cow<'static, str>;

mod vnode;
mod vtext;
mod velement;
mod vlist;

pub fn h<T: Into<VNode>>(node_like: T) -> VNode {
    node_like.into()
}