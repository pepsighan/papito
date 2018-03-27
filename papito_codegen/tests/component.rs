#![feature(proc_macro, conservative_impl_trait)]

extern crate papito;
#[macro_use]
extern crate papito_dom;
#[macro_use]
extern crate papito_codegen;
#[macro_use]
extern crate stdweb;

use papito::prelude::{Lifecycle, Render};
use papito_dom::prelude::VNode;
use papito_codegen::{component, render, events, event};
use stdweb::web::event::ClickEvent;

#[test]
fn should_impl_button_component() {
    #[component]
    struct Button;

    impl Lifecycle for Button {}
    #[render]
    impl Render for Button {
        fn render(&self) -> VNode {
            h!("button", h!("Click"))
        }
    }

    h!(comp Button);
}

#[test]
fn should_derive_default_lifecycle() {
    #[derive(Lifecycle)]
    #[component]
    struct Button;

    #[render]
    impl Render for Button {
        fn render(&self) -> VNode {
            h!("button", h!("Click"))
        }
    }

    h!(comp Button);
}

#[test]
fn should_create_event_wrappers() {
    #[derive(Lifecycle)]
    #[component]
    struct Button;

    #[events]
    impl Button {
        #[event]
        fn on_click(&self, _: ClickEvent) {
            console!(log, "Clicked");
        }
    }

    #[render]
    impl Render for Button {
        fn render(&self) -> VNode {
            h!("button", [ self.on_click() ], h!("Click"))
        }
    }

    h!(comp Button);
}