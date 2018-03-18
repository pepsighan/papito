use CowStr;
use std::fmt::Display;
use std::fmt::{Formatter, self};
#[cfg(target_arch = "wasm32")]
use stdweb::web::TextNode;

#[derive(Debug, Eq, PartialEq)]
pub struct VText {
    content: CowStr,
    #[cfg(target_arch = "wasm32")]
    dom_ref: Option<TextNode>
}

impl VText {
    pub fn new(content: CowStr) -> VText {
        VText {
            content,
            #[cfg(target_arch = "wasm32")]
            dom_ref: None
        }
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
