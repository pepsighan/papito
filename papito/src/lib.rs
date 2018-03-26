extern crate papito_dom;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

use papito_dom::prelude::VNode;
use papito_dom::{comp, h, Component, ComponentOf};
#[cfg(target_arch = "wasm32")]
use stdweb::web::{document, Element, INonElementParentNode};
#[cfg(target_arch = "wasm32")]
use papito_dom::{DOMRender, RenderRequest};
use std::ops::Deref;

pub mod prelude {
    pub use papito_dom::{Component, Lifecycle, Render};
}

pub struct App {
    vdom: VNode,
    #[cfg(target_arch = "wasm32")]
    render_req: RenderRequest,
}

impl App {
    pub fn new<C: ComponentOf>() -> App where
        C::Comp: Component<Props=()> + 'static {
        #[cfg(target_arch = "wasm32")]
        js! { @(no_return)
            window.__schedule_papito_render__ = function() {};
        };

        App {
            vdom: h(comp::<C::Comp>(())),
            #[cfg(target_arch = "wasm32")]
            render_req: RenderRequest::new(|| {
                js! { @(no_return)
                    window.__schedule_papito_render__();
                }
            }),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn render<T: Into<AppRoot>>(mut self, app_root: T) {
        let app_root = app_root.into();
        // Re-renders on requests from the components
        let rerender = move |initial_render: bool| {
            if initial_render || self.render_req.receive() {
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
                    setTimeout(function() {
                        rerender(false)
                    });
                }
            };
            // Initial render
            rerender(true);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct AppRoot(Element);

#[cfg(target_arch = "wasm32")]
impl<'a> From<&'a str> for AppRoot {
    fn from(item: &'a str) -> Self {
        AppRoot(document().get_element_by_id(item)
            .expect(&format!("Could not find the element with id `#{}` to hoist the App", item)))
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Element> for AppRoot {
    fn from(item: Element) -> Self {
        AppRoot(item)
    }
}

#[cfg(target_arch = "wasm32")]
impl Deref for AppRoot {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}