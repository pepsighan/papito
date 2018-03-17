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
    ({$($k:expr => $v:expr),*}) => {
        $crate::h($crate::li(vec![ $( ($k, $v) ),* ]))
    };
    ([$($v:expr),*]) => {
        $crate::h($crate::li(vec![ $( $v ),* ]))
    };
    ($n:expr) => {
        $crate::h($crate::txt($n))
    };
    ($n:expr, _) => {
        $crate::h($crate::el(($n, ())))
    };
    ($n:expr, $($m:expr),*) => {
        $crate::h($crate::el(($n, $( $m ),*)))
    };
}

#[cfg(test)]
mod test {
    use vtext::VText;
    use vnode::VNode;
    use velement::VElement;
    use vlist::VList;
    use std::borrow::Cow;

    #[test]
    fn should_create_text_vnode() {
        let node = h!("Hello World");
        assert_eq!(VNode::Text(VText::new("Hello World".into())), node);
    }

    #[test]
    fn should_create_empty_velement() {
        let node = h!("div", _);
        assert_eq!(VNode::Element(VElement::new("div".into(), None, None, None, false)), node);
    }

    #[test]
    fn should_create_texted_velement() {
        let node = h!("span", h!("Hello World"));
        assert_eq!(
            VNode::Element(VElement::new(
                "span".into(),
                None,
                None,
                Some(VNode::Text(VText::new("Hello World".into()))),
                false,
            )),
            node
        );
    }

    #[test]
    fn should_create_self_closing_velement() {
        let node = h!("br", true);
        assert_eq!(
            VNode::Element(VElement::new(
                "br".into(),
                None,
                None,
                None,
                true,
            )),
            node
        );
    }

    #[test]
    fn should_create_vlist() {
        let node = h!({ "1" => h!("div", _), "2" => h!("div", _), "3" => h!("div", _) });
        assert_eq!(
            VNode::List(vec![
                (Cow::from("1"), VNode::Element(VElement::new("div".into(), None, None, None, false))),
                (Cow::from("2"), VNode::Element(VElement::new("div".into(), None, None, None, false))),
                (Cow::from("3"), VNode::Element(VElement::new("div".into(), None, None, None, false))),
            ].into()),
            node
        );
    }

    #[test]
    fn should_create_vlist_without_keys() {
        let node = h!([h!("div", _), h!("div", _), h!("div", _)]);
        assert_eq!(
            VNode::List(vec![
                (Cow::from("0"), VNode::Element(VElement::new("div".into(), None, None, None, false))),
                (Cow::from("1"), VNode::Element(VElement::new("div".into(), None, None, None, false))),
                (Cow::from("2"), VNode::Element(VElement::new("div".into(), None, None, None, false))),
            ].into()),
            node
        );
    }

    #[test]
    fn should_create_velement_with_class() {
        let node = h!("div", vec![("class", "container")]);
        assert_eq!(
            VNode::Element(VElement::new("div".into(), Some("container".into()), None, None, false)),
            node
        );
    }
}
