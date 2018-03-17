use vnode::VNode;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use indexmap::IndexMap;
use CowStr;

type Key = CowStr;

pub struct VList {
    children: IndexMap<Key, VNode>
}

impl VList {
    pub fn new(children: IndexMap<CowStr, VNode>) -> VList {
        VList {
            children
        }
    }
}

impl Display for VList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (_, v) in self.children.iter() {
            write!(f, "{}", v)?;
        }
        Ok(())
    }
}

impl<T: Into<CowStr>> From<Vec<(T, VNode)>> for VList {
    fn from(item: Vec<(T, VNode)>) -> Self {
        let children = item.into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect();
        VList::new(children)
    }
}
