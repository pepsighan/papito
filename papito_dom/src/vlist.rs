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
    use vdiff::DOMReorder;
    use vdiff::NextDOMNode;
    use stdweb::web::Node;

    impl DOMPatch<VList> for VList {
        fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnodes: Option<&mut VList>) {
            if let Some(old_vnodes) = old_vnodes {
                let mut patched_node_keys = vec![];
                {
                    let mut next_node = None;
                    for (k, v) in self.children.iter_mut().rev() {
                        if let Some(pre_vnode) = old_vnodes.children.get_mut(k) {
                            // Patch if any old VNode found
                            v.patch(parent, next_node.as_ref(), Some(pre_vnode));
                            patched_node_keys.push(k.clone());
                        } else {
                            v.patch(parent, next_node.as_ref(), None);
                        }
                        // should rename it to dom_node()
                        next_node = v.next_dom_node();
                    }
                }
                let mut next_key = None;
                for (k, new_node) in self.children.iter().rev() {
                    let new_pos = self.position(k);
                    let old_pos = old_vnodes.position(k);
                    if old_pos.is_none() {
                        // It is a new node and already inserted to the write place.
                    } else if new_pos.unwrap() != old_pos.unwrap() {
                        if let Some(next_key) = next_key {
                            // This node can be placed before the next one.
                            let next_vnode = self.children.get(next_key).unwrap();
                            new_node.move_before(parent, &next_vnode.next_dom_node().unwrap());
                        } else {
                            // since there is no node after it, do nothing. as all is alright.
                        }
                    }
                    next_key = Some(k);
                }
                for (k, v) in old_vnodes.children.iter_mut() {
                    if patched_node_keys.iter().position(|it| it == k).is_none() {
                        v.remove(parent);
                    }
                }
            } else {
                for (_, v) in self.children.iter_mut() {
                    v.patch(parent, None, None);
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
        fn move_to_last(&self, parent: &Element) {
            for (_, v) in self.children.iter() {
                v.move_to_last(parent);
            }
        }

        fn move_before(&self, parent: &Element, next: &Node) {
            for (_, v) in self.children.iter() {
                v.move_before(parent, next);
            }
        }
    }

    impl NextDOMNode for VList {
        fn next_dom_node(&self) -> Option<Node> {
            self.children.iter().next().and_then(|it| it.1.next_dom_node())
        }
    }
}