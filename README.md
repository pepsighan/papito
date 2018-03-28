# Papito (पपितो) = Papaya

[![Build Status](https://travis-ci.org/csharad/papito.svg?branch=master)](https://travis-ci.org/csharad/papito)

A Vue & React inspired Frontend Web Framework in Rust for the WASM platform. Developed by a beginner for beginners.

### Goals

+ Beginner Friendly. Draws cues from Vue and React.
+ Pure Rust web apps.
+ Cargo only (without webpack). Should provide optional tools that mimic loaders such as `file-loader`, `style-loader`, `url-loader`.

It is still under active development. So tread carefully.

### Demo

```rust
#![feature(proc_macro, conservative_impl_trait)]

#[macro_use]
extern crate papito_codegen;
#[macro_use]
extern crate papito_dom;
extern crate papito;
#[macro_use]
extern crate stdweb;

use papito_codegen::{component, render, events, event};
use papito::prelude::{Render, Lifecycle};
use papito_dom::prelude::VNode;
use papito::App;
use stdweb::web::event::ClickEvent;

#[derive(Lifecycle)]
#[component]
struct Div;

#[render]
impl Render for Div {
    fn render(&self) -> VNode {
        h!("div", { "style" => "background-color: #fafafa; color: #666;" },
            h!([
                h!("This is the front page of web."),
                h!(comp Button, { disabled => false })
            ]))
    }
}

#[component]
struct Button {
    #[prop]
    disabled: bool,
    click: bool
}

#[events]
impl Button {
    #[event]
    fn on_click(&mut self, _: ClickEvent) {
        let inv = !self.click;
        self.set_click(inv);
    }
}

impl Lifecycle for Button {
    fn created(&mut self) {
        console!(log, &format!("Initial click value: {}", self.click));
    }

    fn updated(&mut self) {
        console!(log, &format!("You clicked the button: {}", self.click));
    }
}

#[render]
impl Render for Button {
    fn render(&self) -> VNode {
        let click = self.inner.borrow().click;
        let disabled = self.inner.borrow().disabled;
        h!([
            h!("h1", h!("Hello World!")),
            h!("button", { "disabled" => disabled.to_string() }, [ self.on_click() ], h!("Click")),
            h!("h3", h!(if click { "You clicked" } else { "You unclicked" }))
        ])
    }
}

fn main() {
    App::new::<Div>().render("app");
}
```

### Features Supported

* [x] Component props
* [ ] Component Events
* [x] DOM Events
* [x] Reactive states
* [x] Component Lifecycle (only some)
* [x] Server Renderer
* [x] Hyperscript macro h!
* [ ] Vue-like template syntax
* [ ] Context API?