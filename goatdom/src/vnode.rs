use velement::VElement;
use vlist::VList;
use vtext::VText;
use std::fmt::Display;
use std::fmt::{Formatter, self};

pub enum VNode {
    Text(VText),
    Element(VElement),
    List(VList)
}

impl Display for VNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            VNode::Text(ref text) => write!(f, "{}", text),
            VNode::Element(ref element) => write!(f, "{}", element),
            VNode::List(ref list) => write!(f, "{}", list)
        }
    }
}