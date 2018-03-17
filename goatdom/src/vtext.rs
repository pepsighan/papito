use CowStr;
use std::fmt::Display;
use std::fmt::{Formatter, self};

pub struct VText {
    content: CowStr
}

impl VText {
    pub fn new(content: CowStr) -> VText {
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