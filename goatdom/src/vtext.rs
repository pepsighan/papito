use CowStr;
use std::fmt::Display;
use std::fmt::{Formatter, self};

pub struct VText {
    content: CowStr
}

impl VText {
    fn new(content: CowStr) -> VText {
        VText {
            content
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