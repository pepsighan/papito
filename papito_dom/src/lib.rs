extern crate indexmap;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

use std::borrow::Cow;
use vnode::VNode;
use vtext::VText;
use velement::VElement;
use vlist::VList;
#[cfg(target_arch = "wasm32")]
use stdweb::web::event::ConcreteEvent;
use vcomponent::VComponent;

type CowStr = Cow<'static, str>;

mod vnode;
mod vtext;
mod velement;
mod vlist;
mod vcomponent;
#[cfg(target_arch = "wasm32")]
mod vdiff;
#[cfg(target_arch = "wasm32")]
mod events;
mod traits;

#[cfg(target_arch = "wasm32")]
pub use traits::DOMRender;
#[cfg(target_arch = "wasm32")]
pub use events::{DOMEventListener, RenderRequest};
pub use traits::{Render, Component, Lifecycle};

pub mod prelude {
    pub use vnode::VNode;
    #[cfg(not(target_arch = "wasm32"))]
    pub use traits::RenderToString;
}

pub fn comp<T: Component + 'static, P>(props: P) -> VComponent {
    VComponent::new::<T, _>(props)
}

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

#[cfg(target_arch = "wasm32")]
pub fn ev<E, T, F>(listener: E) -> Box<events::DOMEvent> where
    E: Into<events::DOMEventListener<T, F>>,
    F: FnMut(T) + 'static,
    T: ConcreteEvent + 'static {
    Box::new(listener.into())
}

#[macro_export]
macro_rules! h {
    // Creates a component vnode
    (comp $t:tt) => {
        $crate::h($crate::comp::<$t>())
    };
    // Creates vnodes from a vec
    (vec $n:expr) => {
        $crate::h($crate::li($n));
    };
    // Creates keyed vnodes
    ({ $( $k:expr => $v:expr ),* $(,)* }) => {
        $crate::h($crate::li(vec![ $( ($k, $v) ),* ]))
    };
    // Creates default-keyed vnodes
    ([ $( $v:expr ),* $(,)* ]) => {
        $crate::h($crate::li(vec![ $( $v ),* ]))
    };
    // Creates text vnode
    ($n:expr) => {
        $crate::h($crate::txt($n))
    };
    // Creates an empty element
    ($n:expr, _ $(,)*) => {
        $crate::h($crate::el(($n, ())))
    };
    // Creates an element with map based attributes
    ($n:expr, { $($k:expr => $v:expr),* $(,)* } $(,)*) => {
        $crate::h($crate::el(($n, vec![ $( ($k, $v) ),* ])))
    };
    // Creates an element with event handlers
    ($n:expr, [ $( $ev:expr ),* $(,)* ] $(,)*) => {{
        let mut el = $crate::el(($n, ()));
        #[cfg(target_arch = "wasm32")]
        el.set_events(vec![ $( $crate::ev( $ev ) ),* ]);
        $crate::h(el)
    }};
    // Creates an element with map based attributes and event handlers
    ($n:expr, { $($k:expr => $v:expr),* $(,)* }, [ $( $ev:expr ),* $(,)* ] $(,)*) => {{
        let mut el = $crate::el(($n, vec![ $( ($k, $v) ),* ]));
        #[cfg(target_arch = "wasm32")]
        el.set_events(vec![ $( $crate::ev( $ev ) ),* ]);
        $crate::h(el)
    }};
    // Creates an element with map based attributes, event handlers and other arguments
    ($n:expr, { $($k:expr => $v:expr),* $(,)* }, [ $( $ev:expr ),* $(,)* ], $( $o:expr ),* $(,)*) => {{
        let mut el = $crate::el(($n, vec![ $( ($k, $v) ),* ], $( $o ),*));
        #[cfg(target_arch = "wasm32")]
        el.set_events(vec![ $( $crate::ev( $ev ) ),* ]);
        $crate::h(el)
    }};
    // Creates an element with map based attributes along with other arguments
    ($n:expr, { $($k:expr => $v:expr),* $(,)* }, $( $o:expr ),* $(,)*) => {
        $crate::h($crate::el(($n, vec![ $( ($k, $v) ),* ], $( $o ),*)))
    };
    // Creates an element with plain arguments, except attributes (not strictly), and event handlers
    ($n:expr, [ $( $ev:expr ),* $(,)* ], $( $m:expr ),* $(,)*) => {{
        let mut el = $crate::el(($n, $( $m ),*));
        #[cfg(target_arch = "wasm32")]
        el.set_events(vec![ $( $crate::ev( $ev ) ),* ]);
        $crate::h(el)
    }};
    // Creates an element with plain arguments and event handlers
    ($n:expr, $s:expr, [ $( $ev:expr ),* $(,)* ], $( $m:expr ),* $(,)*) => {{
        let mut el = $crate::el(($n, $s, $( $m ),*));
        #[cfg(target_arch = "wasm32")]
        el.set_events(vec![ $( $crate::ev( $ev ) ),* ]);
        $crate::h(el)
    }};
    // Creates an element with plain arguments
    ($n:expr, $( $m:expr ),* $(,)*) => {
        $crate::h($crate::el(($n, $( $m ),*)))
    };
}

#[cfg(test)]
mod test {
    use vtext::VText;
    use vnode::VNode;
    use velement::VElement;
    use std::borrow::Cow;
    #[cfg(target_arch = "wasm32")]
    use stdweb::web::event::InputEvent;
    use traits::{Component, Lifecycle, Render, RenderToString};
    use vcomponent::VComponent;

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
    fn should_create_vlist_from_vec() {
        let list = vec![h!("div", _), h!("div", _), h!("div", _)];
        let node = h!(vec list);
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

    #[test]
    fn should_create_velement_with_class_with_alt_syntax() {
        let node = h!("div", { "class" => "container" });
        assert_eq!(
            VNode::Element(VElement::new("div".into(), Some("container".into()), None, None, false)),
            node
        );
    }

    #[test]
    fn should_create_velement_with_attributes() {
        let node = h!("div", { "style" => "background-color: black;" });
        assert_eq!(
            VNode::Element(VElement::new("div".into(), None, Some(vec![("style", "background-color: black;")].into()), None, false)),
            node
        );
    }

    #[test]
    fn should_create_nested_structure() {
        let node = h!("div", h!("span", _));
        assert_eq!(
            VNode::Element(VElement::new(
                "div".into(),
                None,
                None,
                Some(VNode::Element(VElement::new(
                    "span".into(),
                    None,
                    None,
                    None,
                    false))),
                false)
            ),
            node
        );
    }

    #[test]
    fn should_create_heterogenous_vlist() {
        let node = h!([
            h!("div", _),
            h!("Hello World"),
            h!([
                h!("div", _),
                h!("Hello World"),
            ])
        ]);
        assert_eq!(
            VNode::List(vec![
                VNode::Element(VElement::new("div".into(), None, None, None, false)),
                VNode::Text(VText::new("Hello World".into())),
                VNode::List(vec![
                    VNode::Element(VElement::new("div".into(), None, None, None, false)),
                    VNode::Text(VText::new("Hello World".into()))
                ].into())
            ].into()),
            node
        );
    }

    #[test]
    fn should_create_empty_input_with_event() {
        let node = h!("input", [ |_: InputEvent| {} ]);
        assert_eq!(
            VNode::Element(VElement::new("input".into(), None, None, None, false)),
            node
        );
    }

    #[test]
    fn should_create_empty_input_with_attribute_and_event() {
        let node = h!("input", { "disabled" => "true" }, [ |_: InputEvent| {} ]);
        assert_eq!(
            VNode::Element(VElement::new(
                "input".into(),
                None,
                Some(vec![("disabled", "true")].into()),
                None,
                false)
            ),
            node
        );
    }

    #[test]
    fn should_create_texted_div_with_attribute_and_event() {
        let node = h!("div", { "style" => "color: white;" }, [ |_: InputEvent| {} ], h!("Hello"));
        assert_eq!(
            VNode::Element(VElement::new(
                "div".into(),
                None,
                Some(vec![("style", "color: white;")].into()),
                Some(VNode::Text(VText::new("Hello".into()))),
                false)
            ),
            node
        );
    }

    #[test]
    fn should_create_texted_div_with_event() {
        let node = h!("div", [ |_: InputEvent| {} ], h!("Hello"));
        assert_eq!(
            VNode::Element(VElement::new(
                "div".into(),
                None,
                None,
                Some(VNode::Text(VText::new("Hello".into()))),
                false)
            ),
            node
        );
    }

    #[test]
    fn should_create_a_component() {
        struct Button;

        impl Component for Button {
            fn create(_: Box<Fn()>) -> Self {
                Button
            }
        }

        impl Lifecycle for Button {}
        impl Render for Button {
            fn render(&self) -> VNode {
                h!("button", h!("Click"))
            }
        }

        let node = h!(comp Button);
        assert_eq!(
            VNode::Component(VComponent::new::<Button>()),
            node
        );
    }

    #[test]
    fn should_print_html_for_empty_div() {
        let node = h!("div", _);
        assert_eq!(node.to_string(), "<div></div>");
    }

    #[test]
    fn should_print_html_for_self_closing_br() {
        let node = h!("br", true);
        assert_eq!(node.to_string(), "<br>");
    }

    #[test]
    fn should_print_html_for_texted_div() {
        let node = h!("div", h!("Hello World"));
        assert_eq!(node.to_string(), "<div>Hello World</div>");
    }

    #[test]
    fn should_print_html_for_attributed_button() {
        let node = h!("button", { "class" => "container", "style" => "background-color: black;" }, h!("Click"));
        assert_eq!(node.to_string(), r#"<button class="container" style="background-color: black;">Click</button>"#)
    }

    #[test]
    fn should_print_html_for_nested_spans() {
        let node = h!("span", h!("span", _));
        assert_eq!(node.to_string(), "<span><span></span></span>");
    }

    #[test]
    fn should_print_html_for_ordered_list() {
        let node = h!("ol", h!([
            h!("li", h!("content")),
            h!("li", h!("content")),
            h!("li", h!("content")),
            h!("li", h!("content")),
        ]));
        assert_eq!(node.to_string(), "<ol><li>content</li><li>content</li><li>content</li><li>content</li></ol>");
    }

    #[test]
    fn should_print_html_for_list_of_text() {
        let node = h!("div", h!([
            h!("Hello"),
            h!("Hello"),
            h!("Hello"),
            h!("Hello"),
        ]));
        assert_eq!(node.to_string(), "<div>HelloHelloHelloHello</div>");
    }

    #[test]
    fn should_print_html_for_plain_list() {
        let node = h!([
            h!("div", _),
            h!("div", _),
            h!("div", _),
            h!("div", _),
        ]);
        assert_eq!(node.to_string(), "<div></div><div></div><div></div><div></div>");
    }

    #[test]
    fn should_print_html_for_component() {
        struct Button;

        impl Component for Button {
            fn create(_: Box<Fn()>) -> Self {
                Button
            }
        }

        impl Lifecycle for Button {}
        impl Render for Button {
            fn render(&self) -> VNode {
                h!("button", h!("Click"))
            }
        }

        let mut node = h!(comp Button);
        assert_eq!(node.render_to_string(), "<button>Click</button>");
    }

    #[test]
    fn should_print_html_for_nested_components() {
        struct Button;

        impl Component for Button {
            fn create(_: Box<Fn()>) -> Self {
                Button
            }
        }

        impl Lifecycle for Button {}
        impl Render for Button {
            fn render(&self) -> VNode {
                h!("button", h!("Click"))
            }
        }

        struct Div;

        impl Component for Div {
            fn create(_: Box<Fn()>) -> Self {
                Div
            }
        }

        impl Lifecycle for Div {}
        impl Render for Div {
            fn render(&self) -> VNode {
                h!("div", h!(comp Button))
            }
        }

        let mut node = h!(comp Div);
        assert_eq!(node.render_to_string(), "<div><button>Click</button></div>");
    }
}
