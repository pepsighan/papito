extern crate papito_dom;
#[macro_use]
extern crate stdweb;

use papito_dom::prelude::{Component, VNode, DOMRender, RenderRequest};
use papito_dom::{comp, h};
use stdweb::web::{document, Element, INonElementParentNode};
use std::ops::Deref;

pub struct App {
    vdom: VNode,
    render_req: RenderRequest
}

impl App {
    pub fn new<T: Component + 'static>() -> App {
        js! { @(no_return)
            window.__schedule_papito_render__ = function() {};
        }

        App {
            vdom: h(comp::<T>()),
            render_req: RenderRequest::new(|| {
                js! { @(no_return)
                    window.__schedule_papito_render__();
                }
            })
        }
    }

    pub fn render<T: Into<AppRoot>>(mut self, app_root: T) {
        let app_root = app_root.into();
        // Initial render
        self.vdom.dom_render(&app_root, None, self.render_req.sender());
        // Re-renders on requests from the components
        let rerender = move || {
            if self.render_req.receive() {
                self.vdom.dom_render(&app_root, None, self.render_req.sender());
            }
            js! { @(no_return)
                window.__is_rendering__ = false;
            }
        };
        js! { @(no_return)
            var rerender = @{rerender};
            window.__is_rendering__ = false;
            window.__schedule_papito_render__ = function() {
                if (!window.__is_rendering__) {
                    setTimeout(rerender);
                }
            }
        }
    }
}

pub struct AppRoot(Element);

impl<'a> From<&'a str> for AppRoot {
    fn from(item: &'a str) -> Self {
        AppRoot(document().get_element_by_id(item)
            .expect(&format!("Could not find the element with id `#{}` to hoist the App", item)))
    }
}

impl From<Element> for AppRoot {
    fn from(item: Element) -> Self {
        AppRoot(item)
    }
}

impl Deref for AppRoot {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}