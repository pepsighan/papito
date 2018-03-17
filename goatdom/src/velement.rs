use CowStr;
use indexmap::IndexMap;
use vnode::VNode;

pub struct ClassString(CowStr);

pub struct Attributes(IndexMap<CowStr, CowStr>);

pub struct VElement {
    tag: CowStr,
    class: ClassString,
    attrs: Attributes,
    child: Option<Box<VNode>>
}