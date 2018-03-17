extern crate indexmap;

use std::borrow::Cow;
use vnode::VNode;
use vtext::VText;
use velement::VElement;
use vlist::VList;

type CowStr = Cow<'static, str>;

mod vnode;
mod vtext;
mod velement;
mod vlist;

pub fn txt<T: Into<VText>>(txt: T) -> VText {
    txt.into()
}

pub fn el<T: Into<VElement>>(el: T) -> VElement {
    el.into()
}

pub fn li<T: Into<VList>>(li: T) -> VList {
    li.into()
}

pub fn h<T: Into<VNode>>(node_like: T) -> VNode {
    node_like.into()
}

#[macro_export]
macro_rules! h {
    ([$($n:expr),*]) => {
        $crate::h($crate::li(vec![$( $n ),*]))
    };
    ($n:expr) => {
        $crate::h($crate::txt($n))
    };
    ($n:expr, $($m:expr),*) => {
        $crate::h($crate::el(($n, $( $m ),*)))
    };
}

#[cfg(test)]
mod test {
    use vtext::VText;
    use vnode::VNode;

    #[test]
    fn should_create_text_vnode() {
        let node = h!("Hello World");
        assert_eq!(VNode::Text(VText::new("Hello World".into())), node);
    }
}