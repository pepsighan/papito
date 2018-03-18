use vnode::VNode;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use indexmap::IndexMap;
use CowStr;

type Key = CowStr;

#[derive(Debug, Eq, PartialEq)]
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

impl From<Vec<VNode>> for VList {
    fn from(item: Vec<VNode>) -> Self {
        let children = item.into_iter()
            .enumerate()
            .map(|(k, v)| (k.to_string().into(), v))
            .collect();
        VList::new(children)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::VList;
    use vdiff::{DOMPatch, DOMRemove, DOMReorder};
    use stdweb::web::Element;
    use indexmap::IndexMap;

    impl DOMPatch<VList> for VList {
        fn patch(&mut self, parent: &Element, old_vnode: Option<&VList>) {
            if let Some(ref old_vnode) = old_vnode {
                let mut old_children: IndexMap<_, _> = old_vnode.children.iter().collect();
                for (k, v) in self.children.iter_mut() {
                    if let Some(pre_vnode) = old_children.swap_remove(k) {
                        // Patch if any old VNode found
                        v.patch(parent, Some(pre_vnode));
                        // Reorder based on insertion
                        v.reorder(parent);
                    } else {
                        v.patch(parent, None);
                    }
                }
                // Remove any VNodes left out
                for (_, v) in old_children {
                    v.remove(parent);
                }
            } else {
                for (_, v) in self.children.iter_mut() {
                    v.patch(parent, None);
                }
            }
        }
    }

    impl DOMRemove for VList {
        fn remove(&self, parent: &Element) {
            for (_, child) in self.children.iter() {
                child.remove(parent);
            }
        }
    }

    impl DOMReorder for VList {
        fn reorder(&self, parent: &Element) {
            for (_, v) in self.children.iter() {
                v.reorder(parent);
            }
        }
    }
}