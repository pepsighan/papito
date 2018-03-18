use CowStr;
use std::fmt::{self, Formatter};
use std::fmt::Display;
#[cfg(target_arch = "wasm32")]
use stdweb::web::{Element, TextNode, document, INode};
#[cfg(target_arch = "wasm32")]
use vdiff::VDiff;
use vnode::VNode;

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
impl VDiff for VText {
    type VNodeLike = VText;

    fn apply(&mut self, parent: &Element, old_vnode: Option<&VText>) {
        if let Some(old_vnode) = old_vnode {
            let text_node = old_vnode.dom_ref().unwrap().clone();
            text_node.set_text_content(&self.content);
            self.dom_ref = Some(text_node);
        } else {
            let text_node = document().create_text_node(&self.content);
            self.dom_ref = Some(text_node);
            parent.append_child(self.dom_ref().unwrap());
        }
    }
}
