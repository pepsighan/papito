use CowStr;
use std::fmt::Display;
use std::fmt::{Formatter, self};

pub struct VText {
    content: CowStr
}

impl Display for VText {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}