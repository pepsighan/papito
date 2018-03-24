use vnode::VNode;
use std::fmt::Display;
use std::fmt::{Formatter, self};
use indexmap::IndexMap;
use CowStr;
#[cfg(not(target_arch = "wasm32"))]
use traits::ServerRender;

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

    #[cfg(target_arch = "wasm32")]
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

#[cfg(not(target_arch = "wasm32"))]
impl ServerRender for VList {
    fn server_render(&mut self) {
        for (_, child) in self.children.iter_mut() {
            child.server_render();
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::VList;
    use vdiff::{DOMPatch, DOMRemove};
    use stdweb::web::Element;
    use vdiff::DOMReorder;
    use vdiff::DOMNode;
    use stdweb::web::Node;
    use CowStr;
    use traits::DOMRender;
    use events::RenderRequestSender;
    use indexmap::IndexMap;

    impl DOMPatch<VList> for VList {
        fn patch(mut self, parent: &Element, next: Option<&Node>, old_vnodes: Option<VList>, render_req: RenderRequestSender) -> Self {
            if let Some(mut old_vnodes) = old_vnodes {
                let old_children_pos: IndexMap<CowStr, usize> = old_vnodes.children.iter()
                    .enumerate()
                    .map(|(pos, (k, _))| (k.clone(), pos))
                    .collect();
                let mut next_node = next.map(|it| it.clone());
                let mut children = IndexMap::new();
                for (k, v) in self.children.into_iter().rev() {
                    let v = if let Some(mut pre_vnode) = old_vnodes.children.swap_remove(&k) {
                        // Patch if any old VNode found
                        v.patch(parent, next_node.as_ref(), Some(pre_vnode), render_req.clone())
                    } else {
                        v.patch(parent, next_node.as_ref(), None, render_req.clone())
                    };
                    next_node = v.dom_node();
                    children.insert(k, v);
                }
                self.children = children.into_iter().rev().collect();
                if has_dirty_order(&self, &old_children_pos) {
                    update_positions(&self, parent, &old_children_pos);
                }
                remove_old_vnodes(old_vnodes, parent);
            } else {
                let mut children = IndexMap::new();
                for (k, v) in self.children {
                    let v = v.patch(parent, None, None, render_req.clone());
                    children.insert(k, v);
                }
                self.children = children;
            }
            self
        }
    }

    fn has_dirty_order(new_vnodes: &VList, old_nodes: &IndexMap<CowStr, usize>) -> bool {
        let mut old_last_position = 0;
        for (k, _) in new_vnodes.children.iter() {
            let old_pos = if let Some(pos) = old_nodes.get(k) {
                *pos
            } else {
                // new nodes not considered for order
                continue;
            };
            if old_pos >= old_last_position {
                old_last_position = old_pos;
            } else {
                return true;
            }
        }
        false
    }

    fn update_positions(new_vnodes: &VList, parent: &Element, old_vnodes: &IndexMap<CowStr, usize>) {
        let mut next_key = None;
        for (k, new_node) in new_vnodes.children.iter().rev() {
            let new_pos = new_vnodes.position(k);
            let old_pos = old_vnodes.get(k).map(|it| *it);
            if old_pos.is_none() {
                // It is a new node and already inserted to the write place.
            } else if new_pos.unwrap() != old_pos.unwrap() {
                if let Some(next_key) = next_key {
                    let next_vnode = new_vnodes.children.get(next_key).unwrap();
                    new_node.move_before(parent, &next_vnode.dom_node().unwrap());
                } else {
                    new_node.move_to_last(parent);
                }
            }
            next_key = Some(k);
        }
    }

    fn remove_old_vnodes(old_vnodes: VList, parent: &Element) {
        for (_, v) in old_vnodes.children {
            v.remove(parent);
        }
    }

    impl DOMRemove for VList {
        fn remove(self, parent: &Element) {
            for (_, child) in self.children {
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

    impl DOMNode for VList {
        fn dom_node(&self) -> Option<Node> {
            self.children.iter().next().and_then(|it| it.1.dom_node())
        }
    }

    impl DOMRender for VList {
        fn dom_render(&mut self, parent: &Element, next: Option<&Node>, render_req: RenderRequestSender) {
            for (_, child) in self.children.iter_mut() {
                child.dom_render(parent, next, render_req.clone());
            }
        }
    }
}
