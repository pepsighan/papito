extern crate papito_dom;
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
        App {
            vdom: h(comp::<T>()),
            render_req: RenderRequest::new()
        }
    }

    pub fn render<T: Into<AppRoot>>(&mut self, app_root: T) {
        let app_root = app_root.into();
        self.vdom.dom_render(&app_root, None);
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