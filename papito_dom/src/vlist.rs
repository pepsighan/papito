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

    impl DOMPatch<VList> for VList {
        fn patch(&mut self, parent: &Element, next: Option<&Node>, old_vnodes: Option<&mut VList>, render_req: RenderRequestSender) {
            if let Some(old_vnodes) = old_vnodes {
                let mut patched_node_keys = vec![];
                {
                    let mut next_node = next.map(|it| it.clone());
                    for (k, v) in self.children.iter_mut().rev() {
                        if let Some(pre_vnode) = old_vnodes.children.get_mut(k) {
                            // Patch if any old VNode found
                            v.patch(parent, next_node.as_ref(), Some(pre_vnode), render_req.clone());
                            patched_node_keys.push(k.clone());
                        } else {
                            v.patch(parent, next_node.as_ref(), None, render_req.clone());
                        }
                        next_node = v.dom_node();
                    }
                }
                if has_dirty_order(&self, &old_vnodes) {
                    update_positions(&self, parent, &old_vnodes);
                }
                remove_old_vnodes(old_vnodes, patched_node_keys, parent);
            } else {
                for (_, v) in self.children.iter_mut() {
                    v.patch(parent, None, None, render_req.clone());
                }
            }
        }
    }

    fn has_dirty_order(new_vnodes: &VList, old_nodes: &VList) -> bool {
        let mut old_last_position = 0;
        for (k, _) in new_vnodes.children.iter() {
            let old_pos = if let Some(pos) = old_nodes.position(k) {
                pos
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

    fn update_positions(new_vnodes: &VList, parent: &Element, old_vnodes: &VList) {
        let mut next_key = None;
        for (k, new_node) in new_vnodes.children.iter().rev() {
            let new_pos = new_vnodes.position(k);
            let old_pos = old_vnodes.position(k);
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

    fn remove_old_vnodes(old_vnodes: &mut VList, patched_node_keys: Vec<CowStr>, parent: &Element) {
        for (k, v) in old_vnodes.children.iter_mut() {
            if patched_node_keys.iter().position(|it| it == k).is_none() {
                v.remove(parent);
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
