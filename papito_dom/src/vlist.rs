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

    fn position(&self, key: &str) -> Option<usize> {
        self.children.iter().position(|(k, _)| k == key)
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
    use vdiff::{DOMPatch, DOMRemove};
    use stdweb::web::Element;
    use indexmap::IndexMap;
    use stdweb::web::INode;
    use vdiff::DOMReorder;

    impl DOMPatch<VList> for VList {
        fn patch(&mut self, parent: &Element, old_vnodes: Option<&mut VList>) {
            if let Some(old_vnodes) = old_vnodes {
                let mut added_node_keys = vec![];
                let mut patched_node_keys = vec![];
                {
                    for (k, v) in self.children.iter_mut() {
                        if let Some(pre_vnode) = old_vnodes.children.get_mut(k) {
                            // Patch if any old VNode found
                            v.patch(parent, Some(pre_vnode));
                            patched_node_keys.push(k.clone());
                        } else {
                            added_node_keys.push(k.clone());
//                            v.patch(parent, None);
                        }
                    }
                    // Remove any VNodes left out
//                    for (_, v) in old_children {
//                        v.remove(parent);
//                    }
                }
//                let mut reorder_forced = false;
//                for (k, v) in self.children.iter() {
//                    if reorder_forced || self.position(k) != old_vnodes.position(k) {
//                        v.reorder(parent);
//                        if !reorder_forced {
//                            reorder_forced = true;
//                        }
//                    }
//                }
            } else {
                for (_, v) in self.children.iter_mut() {
                    v.patch(parent, None);
                }
            }
        }
    }

    impl DOMRemove for VList {
        fn remove(&mut self, parent: &Element) {
            for (_, child) in self.children.iter_mut() {
                child.remove(parent);
            }
        }
    }

    impl DOMReorder for VList {
        fn append_child(&self, parent: &Element) {
            for (_, v) in self.children.iter() {
                v.append_child(parent);
            }
        }

        fn insert_before<T: INode>(&self, parent: &Element, next: &T) {
            for (_, v) in self.children.iter() {
                v.insert_before(parent, next);
            }
        }
    }
}