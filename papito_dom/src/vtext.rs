use CowStr;
use std::fmt::{self, Formatter};
use std::fmt::Display;
#[cfg(target_arch = "wasm32")]
use stdweb::web::TextNode;

#[derive(Debug, Eq, PartialEq)]
pub struct VText {
    content: CowStr,
    #[cfg(target_arch = "wasm32")]
    dom_ref: Option<TextNode>,
}

impl VText {
    pub fn new(content: CowStr) -> VText {
        VText {
            content,
            #[cfg(target_arch = "wasm32")]
            dom_ref: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn dom_ref(&self) -> Option<&TextNode> {
        self.dom_ref.as_ref()
    }
}

impl Display for VText {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl<T: Into<CowStr>> From<T> for VText {
    fn from(item: T) -> Self {
        VText::new(item.into())
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use stdweb::web::{Element, document, INode};
    use vdiff::{DOMPatch, DOMRemove};
    use super::VText;
    use vdiff::DOMReorder;
    use vdiff::NextDOMNode;
    use stdweb::web::Node;

    impl DOMPatch<VText> for VText {
        fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnode: Option<&mut VText>) {
            if let Some(old_vnode) = old_vnode {
                let text_node = old_vnode.dom_ref().unwrap().clone();
                if old_vnode.content != self.content {
                    text_node.set_text_content(&self.content);
                }
                self.dom_ref = Some(text_node);
            } else {
                let text_node = document().create_text_node(&self.content);
                self.dom_ref = Some(text_node);
                if let Some(next) = next {
                    parent.insert_before(self.dom_ref().unwrap(), next);
                } else {
                    parent.append_child(self.dom_ref().unwrap());
                }
            }
        }
    }

    impl DOMReorder for VText {
        fn move_to_last(&self, parent: &Element) {
            let dom_ref = self.dom_ref().expect("Cannot append previously non-existent text node.");
            parent.append_child(dom_ref);
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            parent.insert_before(self.dom_ref().expect("Cannot insert previously non-existent text node."), next)
                .unwrap();
        }
    }

    impl DOMRemove for VText {
        fn remove(&mut self, parent: &Element) {
            parent.remove_child(&self.dom_ref.take()
                .expect("Cannot remove non-existent text node.")
            ).unwrap();
        }
    }

    impl NextDOMNode for VText {
        fn next_dom_node(&self) -> Option<Node> {
            self.dom_ref.clone().map(|it| it.into())
        }
    }
}