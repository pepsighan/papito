use vnode::VNode;
use std::fmt::Display;
use std::fmt::{Formatter, self};

pub struct VList {
    children: Vec<VNode>
}

impl VList {
    pub fn new(children: Vec<VNode>) -> VList {
        VList {
            children
        }
    }
}

impl Display for VList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for child in self.children.iter() {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}